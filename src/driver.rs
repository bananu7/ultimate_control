use std::io::Read;
use std::net::TcpStream;
use crate::packet::*;
use std::ascii::escape_default;

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
                mute
            ),
            &mut self.stream
        )
    }

    pub fn read_response(&mut self) -> std::io::Result<()> {
        
        fn _show(bs: &[u8]) -> String {
            let mut visible = String::new();
            for &b in bs {
                let part: Vec<u8> = escape_default(b).collect();
                visible.push_str(std::str::from_utf8(&part).unwrap());
            }
            visible
        }

        let mut buffer = [0; 3000];

        let nbytes = self.stream.read(&mut buffer)?;

        /*if nbytes == 0 {
            return Ok(());
        }*/

        println!("{}", nbytes);
        //println!("{}", show(&buffer));

        Ok(())
    }
}