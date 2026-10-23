use std::io::{Error, ErrorKind};
use std::string::ToString;

use futures::StreamExt;
use futures::sink::SinkExt;
use futures::stream::SplitStream;

use tokio::net::{TcpStream, UdpSocket};
use tokio::sync::mpsc;
use tokio::time::{Duration, Instant, interval, timeout};
use tokio_util::codec::Framed;

use crate::async_packet::PacketCodec;
use crate::semantic::{Param, params_from_zm, print_packet};
use crate::types::{AddressPair, UcPacket};

/// Universal Control hosts a UCNET server for USB devices on this port.
pub const DEFAULT_ADDR: &str = "127.0.0.1:49670";

/// How often we send a keep-alive. The reference StudioLive client uses 1 s
/// and treats 3 s of silence as a dead connection.
const KEEPALIVE_PERIOD: Duration = Duration::from_secs(1);

pub struct AsyncUcDriver {
    reader: SplitStream<Framed<TcpStream, PacketCodec>>,
    cmd_tx: mpsc::Sender<UcPacket>,
    udp_port: u16,
    /// Echo outgoing packets to stdout.
    pub trace: bool,
}

fn to_io_err(e: impl ToString) -> std::io::Error {
    Error::other(e.to_string())
}

impl AsyncUcDriver {
    /// Connect to Universal Control on the local machine.
    pub async fn new() -> std::io::Result<AsyncUcDriver> {
        Self::connect(DEFAULT_ADDR).await
    }

    pub async fn connect(addr: &str) -> std::io::Result<AsyncUcDriver> {
        let stream = TcpStream::connect(addr).await?;

        let codec = PacketCodec {};
        let (mut writer, reader) = Framed::new(stream, codec).split();

        let (tx, mut rx) = mpsc::channel(100);

        // spawn hb task
        let hb_tx = tx.clone();
        tokio::spawn(async move {
            let mut interval = interval(KEEPALIVE_PERIOD);

            loop {
                interval.tick().await;
                let res = hb_tx
                    .send(UcPacket::KA(AddressPair { a: 0x6b, b: 0x66 }))
                    .await;

                // explicit break to annotate return type
                // and avoid turbofish "unreachable"
                if let Some(e) = res.err() {
                    break to_io_err(e);
                }
            }
        });

        // spawn sender task
        tokio::spawn(async move {
            while let Some(packet) = rx.recv().await {
                writer.send(packet).await?;
            }
            Ok::<(), std::io::Error>(())
        });

        let udp_socket = UdpSocket::bind("127.0.0.1:0").await?;
        let udp_port = udp_socket.local_addr()?.port();

        let mut buf = [0; 1024];
        // spawn udp reader task
        tokio::spawn(async move {
            while let Ok((len, addr)) = udp_socket.recv_from(&mut buf).await {
                println!("{:?} bytes received from {:?}", len, addr);
            }
            Ok::<(), std::io::Error>(())
        });

        Ok(AsyncUcDriver {
            reader,
            cmd_tx: tx,
            udp_port,
            trace: true,
        })
    }

    async fn send(&mut self, packet: UcPacket) -> std::io::Result<()> {
        if self.trace {
            print_packet(&packet, true);
        }
        self.cmd_tx.send(packet).await.map_err(to_io_err)
    }

    // Notify the device about UDP port number to send subscription data to
    async fn um(&mut self, port_number: u16) -> std::io::Result<()> {
        self.send(UcPacket::UM {
            ap: AddressPair { a: 0, b: 0x66 },
            udp_port: port_number,
        })
        .await
    }

    pub async fn subscribe(&mut self) -> std::io::Result<()> {
        if self.trace {
            println!("Subscribing on UDP port {}", self.udp_port);
        }
        self.um(self.udp_port).await?;

        let sub_msg_uc = r#"{"id": "Subscribe","clientName": "Universal Control","clientInternalName": "ucapp","clientType": "PC","clientDescription": "DESKTOP-VLS3MG2","clientIdentifier": "DESKTOP-VLS3MG2","clientOptions": "perm users","clientEncoding": 23117}"#;
        self.send(UcPacket::JM(
            AddressPair { a: 0x6b, b: 0x66 },
            sub_msg_uc.to_string(),
        ))
        .await
    }

    /// Set any numeric parameter, e.g. `main/ch1/hardwareMute` to `1.0`.
    pub async fn set_param(&mut self, name: &str, value: f32) -> std::io::Result<()> {
        self.send(UcPacket::PV(
            AddressPair { a: 0x6b, b: 0x66 },
            name.to_string(),
            value,
        ))
        .await
    }

    /// Set a boolean parameter (`1.0` on, `0.0` off).
    pub async fn set_bool(&mut self, name: &str, on: bool) -> std::io::Result<()> {
        self.set_param(name, if on { 1.0 } else { 0.0 }).await
    }

    /// Set a parameter and wait for the device to echo it back, which it does
    /// for every PV it accepts. Returns the echoed value.
    pub async fn set_param_confirmed(
        &mut self,
        name: &str,
        value: f32,
        wait: Duration,
    ) -> std::io::Result<f32> {
        self.set_param(name, value).await?;

        let deadline = Instant::now() + wait;
        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            let next = timeout(remaining, self.next_packet()).await.map_err(|_| {
                Error::new(
                    ErrorKind::TimedOut,
                    format!("device did not echo {name} within {wait:?}"),
                )
            })?;
            match next {
                Some(Ok(UcPacket::PV(_, echoed, v))) if echoed == name => return Ok(v),
                Some(Ok(other)) => {
                    if self.trace {
                        print_packet(&other, false);
                    }
                }
                Some(Err(e)) => return Err(e),
                None => return Err(Error::new(ErrorKind::ConnectionAborted, "stream ended")),
            }
        }
    }

    pub async fn ch1_mute(&mut self, mute: bool) -> std::io::Result<()> {
        self.set_bool(crate::params::CH1_MUTE, mute).await
    }

    pub async fn ch1_volume(&mut self, volume: f32) -> std::io::Result<()> {
        self.set_param(crate::params::CH1_VOLUME, volume).await
    }

    /// Software mute of the main bus (not exposed in the UC window).
    pub async fn main_mute(&mut self, mute: bool) -> std::io::Result<()> {
        self.set_bool(crate::params::MAIN_MUTE, mute).await
    }

    /// The mute the physical button on the device toggles.
    pub async fn hardware_mute(&mut self, mute: bool) -> std::io::Result<()> {
        self.set_bool(crate::params::HARDWARE_MUTE, mute).await
    }

    /// Next packet from the device. `None` means the connection is gone
    /// (closed by the peer, or the framing got out of sync).
    pub async fn next_packet(&mut self) -> Option<std::io::Result<UcPacket>> {
        self.reader.next().await
    }

    /// Wait for the `Synchronize` state dump the device sends after a
    /// subscription and return it as flat `path = value` parameters.
    pub async fn wait_for_state(&mut self, wait: Duration) -> std::io::Result<Vec<Param>> {
        let deadline = Instant::now() + wait;
        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            let next = timeout(remaining, self.next_packet())
                .await
                .map_err(|_| Error::new(ErrorKind::TimedOut, "no state dump received"))?;
            match next {
                Some(Ok(UcPacket::ZM {
                    compressed_payload, ..
                })) => {
                    let params = params_from_zm(&compressed_payload)?;
                    // The device sends several ZMs; only the Synchronize one
                    // carries the parameter tree.
                    if !params.is_empty() {
                        return Ok(params);
                    }
                }
                Some(Ok(other)) => {
                    if self.trace {
                        print_packet(&other, false);
                    }
                }
                Some(Err(e)) => return Err(e),
                None => return Err(Error::new(ErrorKind::ConnectionAborted, "stream ended")),
            }
        }
    }

    /// Print everything the device sends until the connection drops.
    pub async fn read_response(&mut self) {
        while let Some(data) = self.next_packet().await {
            match data {
                Ok(packet) => print_packet(&packet, false),
                Err(e) => {
                    println!("Packet parsing failed! - {}", e);
                    break;
                }
            }
        }
    }
}
