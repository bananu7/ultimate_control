use std::ascii::escape_default;
use std::io::prelude::*;
use std::net::TcpStream;

pub mod header;
pub mod packet;

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

    //println!("{}", show(&buffer));
    println!("{}", nbytes);

    return Ok(());
}

fn main() -> std::io::Result<()> {
   
    if let Ok(mut stream) = TcpStream::connect("127.0.0.1:55935") {
        println!("Connected to the server!");

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

        // another subscribe request (282)
        println!("Subsribe_2 ({})", &header::SUBSCRIBE_2.len());
        stream.write(&header::SUBSCRIBE_2)?;
        // big response (2706)
        read(&mut stream)?;

        // then it asks for presets (600)
        // two responses(1671, 1708)

        // then binary_1 again
        stream.write(&header::BINARY_1)?;
        // no response

        // then binary_2 (12 bytes)
        stream.write(&header::BINARY_2)?;
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