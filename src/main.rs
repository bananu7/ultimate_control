use std::ascii::escape_default;
use std::io::prelude::*;
use std::net::TcpStream;

pub mod header;

fn show(bs: &[u8]) -> String {
    let mut visible = String::new();
    for &b in bs {
        let part: Vec<u8> = escape_default(b).collect();
        visible.push_str(std::str::from_utf8(&part).unwrap());
    }
    visible
}

fn main() -> std::io::Result<()> {
   
    if let Ok(mut stream) = TcpStream::connect("127.0.0.1:55935") {
        println!("Connected to the server!");

        stream.write(&header::HEADER)?;

        let mut buffer = [0; 500];
        loop {
            let nbytes = stream.read(&mut buffer)?;
            if nbytes == 0 {
                return Ok(());
            }

            println!("{}", show(&buffer));
        }

    } else {
        println!("Couldn't connect to server...");
    }

    Ok(())
}