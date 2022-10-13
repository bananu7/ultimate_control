use std::ascii::escape_default;
use std::io::prelude::*;
use std::net::TcpStream;

pub mod header;
pub mod packet;
use packet::*;

fn show(bs: &[u8]) -> String {
    let mut visible = String::new();
    for &b in bs {
        let part: Vec<u8> = escape_default(b).collect();
        visible.push_str(std::str::from_utf8(&part).unwrap());
    }
    visible
}

fn read(stream: &mut TcpStream) -> std::io::Result<()> {
    let mut buffer = [0; 3000];

    let nbytes = stream.read(&mut buffer)?;

    /*if nbytes == 0 {
        return Ok(());
    }*/

    println!("{}", nbytes);
    //println!("{}", show(&buffer));

    return Ok(());
}

fn main() -> std::io::Result<()> {
    if let Ok(mut stream) = TcpStream::connect("127.0.0.1:55935") {
        println!("Connected to the server!");

        let mut write = |p: &UcPacket| {
            write_packet(&p, &mut stream)
        };

        /*
        // header - 1
        write_packet(&UcPacket::UM([0x00,0x00,0x65,0x00,0x5e,0xf1]), &mut stream)?;

        let sub_msg = r#"{"id": "Subscribe","clientName": "","clientInternalName": "","clientType": "PC","clientDescription": "DESKTOP","clientIdentifier": "DESKTOP","clientOptions": "","clientEncoding": 23117}"#;        
        write_packet(&UcPacket::JM(
            AddressPair{ a: 0x6a, b: 0x65 },
            sub_msg.to_string()),
        &mut stream)?;
        // Subscription reply (709)
        read(&mut stream)?;

        write_packet(&UcPacket::UM([0x00,0x00,0x67,0x00,0x5e,0xf1]), &mut stream)?;

        write_packet(&UcPacket::JM(
            AddressPair{ a: 0x66, b: 0x67 },
            sub_msg.to_string()),
        &mut stream)?;
        // Subscription reply (709)
        read(&mut stream)?;

        write_packet(&UcPacket::KA(AddressPair{ a: 0x66, b: 0x67 }), &mut stream)?;

        let invoke_method = r#"{"id": "InvokeMethod","url": "","method": "getOptions","callid": 100}"#;
        write_packet(&UcPacket::JM(
            AddressPair{ a: 0x66, b: 0x67 },
            invoke_method.to_string()),
        &mut stream)?;

        write_packet(&UcPacket::KA(AddressPair{ a: 0x6a, b: 0x65 }), &mut stream)?;
        */
        // end of header 1

        // binary_1
        //write_packet(&UcPacket::KA(AddressPair{ a: 0x68, b: 0x67 }), &mut stream)?;
        //write_packet(&UcPacket::KA(AddressPair{ a: 0x6a, b: 0x65 }), &mut stream)?;

        // subscribe_2
        write(&UcPacket::UM([0x00,0x00,0x66,0x00,0x84,0xe8]))?;
        let sub_msg_uc = r#"{"id": "Subscribe","clientName": "Universal Control","clientInternalName": "ucapp","clientType": "PC","clientDescription": "DESKTOP","clientIdentifier": "DESKTOP","clientOptions": "perm users","clientEncoding": 23117}"#;
        write(&UcPacket::JM(
            AddressPair{ a: 0x6b, b: 0x66 },
            sub_msg_uc.to_string()))?;
        // end of subscribe_2

        write(&UcPacket::PV(
            AddressPair{ a: 0x6b, b: 0x66 },
            "line/ch1/mute".to_string(),
            true))?;

        write(&UcPacket::FR(
            AddressPair{ a: 0x6b, b: 0x66 },
            1,
            "Listpresets/channel".to_string()))?;
        // 32 byte response
        read(&mut stream)?;

    } else {
        println!("Couldn't connect to server...");
    }

    Ok(())
}

fn main3() -> std::io::Result<()> {
   
    if let Ok(mut stream) = TcpStream::connect("127.0.0.1:55935") {
        println!("Connected to the server!");

        /*
        // Subscription request (539)
        stream.write(&header::HEADER)?;
        // Subscription reply (709)
        read(&mut stream)?;

        println!("getWDM ({})", &header::INVOKE_WDM_SETUP.len());
        // Invoke - getWDMSetupList (90)
        stream.write(&header::INVOKE_WDM_SETUP)?;
        // Method response (104)
        read(&mut stream)?;

        // no idea what that is (24)
        println!("Binary_1 ({})", &header::BINARY_1.len());
        stream.write(&header::BINARY_1)?;
        // no response

        */

        // another subscribe request (282)
        println!("Subsribe_2 ({})", &header::SUBSCRIBE_2.len());
        stream.write(&header::SUBSCRIBE_2)?;
        // big response (2706)
        read(&mut stream)?;

        // then it asks for presets (600)
        // two responses(1671, 1708)

        // then binary_1 again
        //stream.write(&header::BINARY_1)?;
        // no response

        // then binary_2 (12 bytes)
        //stream.write(&header::BINARY_2)?;
        // 12-byte response
        read(&mut stream)?;


        // then it sends BINARY_1 and BINARY_2 mixed in with mute commands (32)
        stream.write(&header::MUTE)?;
        // 32 byte response
        read(&mut stream)?;

    } else {
        println!("Couldn't connect to server...");
    }

    Ok(())
}