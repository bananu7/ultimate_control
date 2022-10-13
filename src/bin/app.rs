use ultimate_control::types::UcPacket;
use ultimate_control::types::AddressPair;
use tokio_util::codec::Framed;
use tokio::net::TcpStream;
use tokio_stream::StreamExt;
use futures::sink::SinkExt;
use ultimate_control::async_packet::PacketCodec;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let stream = TcpStream::connect("127.0.0.1:55935").await?;

    println!("Connected to the server!");

    let codec = PacketCodec{};
    let mut framed = Framed::new(stream, codec);

    let sub_msg_uc =r#"{"id": "Subscribe","clientName": "Universal Control","clientInternalName": "ucapp","clientType": "PC","clientDescription": "DESKTOP","clientIdentifier": "DESKTOP","clientOptions": "perm users","clientEncoding": 23117}"#;
    framed.send(
        UcPacket::JM (
            AddressPair{ a: 0x6b, b: 0x66 },
            sub_msg_uc.to_string()
        )
    ).await?;
    println!("Sent JM packet");

    framed.send(
        UcPacket::PV (
            AddressPair{ a: 0x6b, b: 0x66 },
            "line/ch1/mute".to_string(),
            true
        )
    ).await?;
    println!("Sent PV packet");

    while let Some(packet) = framed.next().await {
        println!("Received packet {:?}", packet);
    }
        
    Ok(())
}

