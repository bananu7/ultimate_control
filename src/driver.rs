use std::net::TcpStream;
use crate::packet::*;
use crate::types::*;

pub struct UcDriver {
    stream: TcpStream
}

impl UcDriver {
    pub fn new() -> std::io::Result<UcDriver> {
        Ok(UcDriver {
            stream: TcpStream::connect("127.0.0.1:55935")?
        })
    }

    pub fn subscribe(&mut self) -> std::io::Result<()> {
        let sub_msg_uc =r#"{"id": "Subscribe","clientName": "Universal Control","clientInternalName": "ucapp","clientType": "PC","clientDescription": "DESKTOP","clientIdentifier": "DESKTOP","clientOptions": "perm users","clientEncoding": 23117}"#;
        write_packet(
            &UcPacket::JM (
                AddressPair{ a: 0x6b, b: 0x66 },
                sub_msg_uc.to_string()
            ),
            &mut self.stream
        )
    }

    pub fn ch1_mute(&mut self, mute: bool) -> std::io::Result<()>  {
        write_packet(
            &UcPacket::PV (
                AddressPair{ a: 0x6b, b: 0x66 },
                "line/ch1/mute".to_string(),
                if mute { 1.0 } else { 0.0 }
            ),
            &mut self.stream
        )
    }

    pub fn read_response(&mut self) -> std::io::Result<()> {
        match read_packet(&mut self.stream) {
            Ok(p) => println!("{:?}", p),
            Err(e) => println!("wrong response - {}", e),
        }

        Ok(())
    }
}