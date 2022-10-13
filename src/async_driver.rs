use tokio_util::codec::Framed;
use tokio::net::TcpStream;
use tokio_stream::StreamExt;
use futures::sink::SinkExt;

use crate::types::{UcPacket, AddressPair};
use crate::async_packet::PacketCodec;

pub struct AsyncUcDriver {
    framed: Framed<TcpStream, PacketCodec>,
}

impl AsyncUcDriver {
    pub async fn new() -> std::io::Result<AsyncUcDriver> {

        let stream = TcpStream::connect("127.0.0.1:55935").await?;

        let codec = PacketCodec{};
        let framed = Framed::new(stream, codec);

        Ok(AsyncUcDriver {
            framed: framed
        })
    }

    pub async fn subscribe(&mut self) -> std::io::Result<()> {
        let sub_msg_uc =r#"{"id": "Subscribe","clientName": "Universal Control","clientInternalName": "ucapp","clientType": "PC","clientDescription": "DESKTOP","clientIdentifier": "DESKTOP","clientOptions": "perm users","clientEncoding": 23117}"#;
        self.framed.send(
            UcPacket::JM (
                AddressPair{ a: 0x6b, b: 0x66 },
                sub_msg_uc.to_string()
            )
        ).await
    }

    pub async fn ch1_mute(&mut self, mute: bool) -> std::io::Result<()>  {
        self.framed.send(
            UcPacket::PV (
                AddressPair{ a: 0x6b, b: 0x66 },
                "line/ch1/mute".to_string(),
                mute
            )
        ).await
    }

    pub async fn read_response(&mut self) {
        while let Some(packet) = self.framed.next().await {
            println!("Received packet {:?}", packet);
        }
    }

}