use std::io::{Error, ErrorKind};
use std::string::ToString;

use futures::StreamExt;
use futures::stream::SplitStream;
use futures::sink::SinkExt;

use tokio::time::interval;
use tokio::time::Duration;
use tokio_util::codec::Framed;
use tokio::net::TcpStream;

use crate::types::{UcPacket, AddressPair};
use crate::async_packet::PacketCodec;
use crate::semantic::{print_packet};

use tokio::sync::mpsc;

pub struct AsyncUcDriver {
    reader: SplitStream<Framed<tokio::net::TcpStream, PacketCodec>>,
    cmd_tx: mpsc::Sender<UcPacket>,
}
    
fn to_io_err(e: impl ToString) -> std::io::Error {
    Error::new(ErrorKind::Other, e.to_string())
}

impl AsyncUcDriver {
    pub async fn new() -> std::io::Result<AsyncUcDriver> {

        let stream = TcpStream::connect("127.0.0.1:49670").await?;

        let codec = PacketCodec{};
        let (mut writer, reader) = Framed::new(stream, codec).split();

        let (tx, mut rx) = mpsc::channel(100);

        // spawn hb task
        let hb_tx = tx.clone();
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(2));

            loop {
                interval.tick().await;
                let res = hb_tx.send(
                    UcPacket::KA(AddressPair { a: 0x6b, b: 0x66 })
                ).await;

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

        Ok(AsyncUcDriver {
            reader: reader,
            cmd_tx: tx.clone(),
        })
    }

    // Notify the device about UDP port number to send subscription data to
    pub async fn um(&mut self, port_number: u16) -> std::io::Result<()> {
        let port_bytes = port_number.to_le_bytes();
        self.cmd_tx.send(
            UcPacket::UM([0x00,0x00,0x66,0x00,port_bytes[0],port_bytes[1]])
        ).await.map_err(to_io_err)
    }

    pub async fn subscribe(&mut self) -> std::io::Result<()> {
        let sub_msg_uc =r#"{"id": "Subscribe","clientName": "Universal Control","clientInternalName": "ucapp","clientType": "PC","clientDescription": "DESKTOP","clientIdentifier": "DESKTOP","clientOptions": "perm users","clientEncoding": 23117}"#;
        self.cmd_tx.send(
            UcPacket::JM (
                AddressPair{ a: 0x6b, b: 0x66 },
                sub_msg_uc.to_string()
            )
        ).await.map_err(to_io_err)
    }

    pub async fn ch1_mute(&mut self, mute: bool) -> std::io::Result<()> {
        self.cmd_tx.send(
            UcPacket::PV (
                AddressPair{ a: 0x6b, b: 0x66 },
                "line/ch1/mute".to_string(),
                if mute { 1.0 } else { 0.0 }
            )
        ).await.map_err(to_io_err)
    }

    pub async fn ch1_volume(&mut self, volume: f32) -> std::io::Result<()> {
        self.cmd_tx.send(
            UcPacket::PV (
                AddressPair{ a: 0x6b, b: 0x66 },
                "line/ch1/volume".to_string(),
                volume,
            )
        ).await.map_err(to_io_err)
    }

    pub async fn read_response(&mut self) {
        while let Some(data) = self.reader.next().await {
            match data {
                Ok(packet) => print_packet(&packet),
                Err(e) => println!("Packet parsing failed! - {}", e.to_string())
            }
            
        }
    }

}