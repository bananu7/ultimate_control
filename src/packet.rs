use std::io::SeekFrom;
use std::io::Cursor;
use std::io::prelude::*;
use std::io::{Error, ErrorKind};
use std::str;

use crate::types::*;

// those are convenience functions that operate directly on memory
pub fn ser(packet: &UcPacket) -> Vec<u8> {
    let mut c = Cursor::new(Vec::new());
    write_packet(&packet, &mut c).unwrap();
    
    let mut out = Vec::new();
    c.seek(SeekFrom::Start(0)).unwrap();
    c.read_to_end(&mut out).unwrap();
    out
}

pub fn deser(buf: &Vec<u8>) -> UcPacket {
    let mut stream = Cursor::new(&buf);
    read_packet(&mut stream).unwrap()
}

// Those functions operate on synchronous Read/Write objects
fn write_address_pair<Writer: Write>(ap: &AddressPair, w: &mut Writer) -> std::io::Result<()> {
    w.write(&ap.a.to_le_bytes())?;
    w.write(&ap.b.to_le_bytes())?;
    Ok(())
}

pub fn write_packet<Writer: Write>(p: &UcPacket, w: &mut Writer) -> std::io::Result<()>  {
    w.write(&[b'U', b'C', 0, 1])?;

    match p {
        UcPacket::JM(ap, s) => {
            let size: u16 = (s.len() + 10).try_into().unwrap(); // 10 bytes for extra metadata
            w.write(&size.to_le_bytes())?;
            w.write(&[b'J', b'M'])?;
            w.write(&ap.a.to_le_bytes())?;
            w.write(&ap.b.to_le_bytes())?;
            w.write(&(s.len() as u32).to_le_bytes())?;
            w.write(s.as_bytes())?;
        }

        UcPacket::UM{ap, udp_port} => {
            let size: u16 = 8;
            w.write(&size.to_le_bytes())?;
            w.write(&[b'U', b'M'])?;
            write_address_pair(ap, w)?;
            w.write(&udp_port.to_le_bytes())?;
        }

        UcPacket::KA(ap) => {
            let size: u16 = 6;
            w.write(&size.to_le_bytes())?;
            w.write(&[b'K', b'A'])?;
            write_address_pair(ap, w)?;
        }

        UcPacket::PV(ap, name, val) => {
            let size: u16 = ((name.len() + 4) as u16).try_into().unwrap();

            // 7 bytes of extra padding at the end
            w.write(&(size + 2 + 7).to_le_bytes())?;
            w.write(&[b'P', b'V'])?;
            write_address_pair(ap, w)?;

            // parameter name
            w.write(name.as_bytes())?;

            // padding with 3 zeros
            w.write(&[0u8,0,0])?;

            // float
            w.write(&val.to_le_bytes())?;
        }

        UcPacket::FR(ap, some_number, buf) => {
            let size: u16 = ((buf.len() + 8) as u16).try_into().unwrap();
            w.write(&size.to_le_bytes())?;
            w.write(&[b'F', b'R'])?;
            write_address_pair(ap, w)?;
            w.write(&some_number.to_le_bytes())?;
            w.write(buf.as_bytes())?;
        }

        UcPacket::ZM{ap, unknown, compressed_payload} => {
            let size: u16 = ((compressed_payload.len() + 6) as u16).try_into().unwrap();
            w.write(&size.to_le_bytes())?;
            w.write(&[b'Z', b'M'])?;
            write_address_pair(ap, w)?;
            w.write(&unknown.to_le_bytes())?;
            w.write(compressed_payload)?;
        }

        UcPacket::PS(ap, buf) => {
            let size: u16 = ((buf.len() + 6) as u16).try_into().unwrap();
            w.write(&size.to_le_bytes())?;
            w.write(&[b'P', b'S'])?;
            write_address_pair(ap, w)?;
            w.write(buf)?;
        }

        UcPacket::PL(ap, name, names) => {
            let size_of_names: usize = 
                names.into_iter().map(|n|n.len()).sum::<usize>() + // each of the names
                names.len(); // newlines and 0 at the end
            let size: u16 = 6 + (name.len() as u16) + 7 + (size_of_names as u16);
            w.write(&size.to_le_bytes())?;
            w.write(&[b'P', b'L'])?;
            write_address_pair(ap, w)?;
            w.write(name.as_bytes())?;
            w.write(&[0x00u8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00])?;
            for n in names {
                w.write(n.as_bytes())?;
            }
        }
    }

    Ok(())
}

pub fn read_packet<Reader: Read>(stream: &mut Reader) -> std::io::Result<UcPacket> {
    let mut header = [0u8; 6];
    let nbytes = stream.read(&mut header)?;

    if nbytes < 6 {
        return Err(Error::new(ErrorKind::Other, "Packet too small to be valid"));
    }

    if header[0] != b'U' || // 'U'
        header[1] != b'C' || // 'C'
        header[2] != 0u8 ||
        header[3] != 1u8
    {
        return Err(Error::new(ErrorKind::Other, "header magic bytes invalid"));
    }

    let size: u16 = (header[4] as u16) + ((header[5] as u16) << 8);

    let mut buf = Vec::new();
    stream.take(size as u64).read_to_end(&mut buf)?;

    parse_packet_contents(&buf)
}

pub fn parse_packet_contents(bytes: &Vec<u8>) -> std::io::Result<UcPacket> {
    let address_pair = AddressPair {
        a: (bytes[2] as u16) | ((bytes[3] as u16) << 8),
        b: (bytes[4] as u16) | ((bytes[5] as u16) << 8),
    };

    match (bytes[0], bytes[1]) {
        (b'J',b'M') => {
            // ignore 32-bit size
            Ok(UcPacket::JM(address_pair, str::from_utf8(&bytes[10..]).unwrap().to_string()))
        }
        (b'P',b'V') => {
            let data_len = bytes.len() - 7;
            // 803f or 0000
            let f = bytes.len()-4;
            let float_data: [u8; 4] = [bytes[f], bytes[f+1], bytes[f+2], bytes[f+3]];
            let val = f32::from_le_bytes(float_data);

            Ok(UcPacket::PV(
                address_pair,
                str::from_utf8(&bytes[6..data_len]).unwrap().to_string(),
                val
            ))
        }
        (b'U',b'M') => {
            if bytes.len() != 8 {
                return Err(Error::other("Wrong amount of bytes for UM packet"));
            }
            let udp_port = u16::from_le_bytes(bytes[6..8].try_into().unwrap());

            Ok(UcPacket::UM{
                ap: address_pair,
                udp_port: udp_port,
            })
        }
        (b'F',b'R') => {
            let some_number = (bytes[6] as u16) + (bytes[7] as u16) << 8;
            let string_query = str::from_utf8(&bytes[8..]).unwrap().to_string();
            Ok(UcPacket::FR(address_pair, some_number, string_query))
        }
        (b'K',b'A') => {
            Ok(UcPacket::KA(address_pair))
        }
        (b'Z',b'M') => {
            Ok(UcPacket::ZM{
                ap: address_pair,
                unknown: u32::from_le_bytes(bytes[6..10].try_into().unwrap()),
                compressed_payload: bytes[10..].to_vec()
            })
        }
        (b'P',b'S') => {
            Ok(UcPacket::PS(address_pair, bytes[6..].to_vec()))
        }
        (b'P',b'L') => {
            // take AddressPair
            // parse key until 7 zeroes
            let mut key = String::new();
            let mut lastbyte = 6;
            for i in lastbyte .. bytes.len() {
                if bytes[i] == 0x00 {
                    // skip 7 zeroes
                    lastbyte += 6;
                    break;
                } else {
                    key.push(bytes[i] as char);
                }
            }
            // parse each item until newline or 0 character
            let mut names = Vec::new();
            let mut name = String::new();
            for i in lastbyte .. bytes.len() {
                if bytes[i] == 0x00 { // end of packet
                    names.push(name);
                    break;
                } else if bytes[i] == 0x0a { // newline
                    names.push(name);
                    name = String::new();
                } else {
                    name.push(bytes[i] as char)
                }
            }

            // make sure all bytes were used
            Ok(UcPacket::PL(address_pair, key, names))
        }
        _ => {
            println!("Unknown packet encountered - {}{}", bytes[0], bytes[1]);
            println!("{:02X?}", bytes[2..].to_vec());
            Err(Error::new(ErrorKind::Other, "Invalid packet"))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn packet_um() {
        let buf = vec![
            // UC01
            0x55,0x43,0x00,0x01,
            // size
            0x08,0x00,
            // UM
            0x55,0x4d,
            // 6 bytes data
            0x00,0x00,0x65,0x00,0x5e,0xf1,
        ];

        let packet = UcPacket::UM{ 
            ap: AddressPair{a: 0, b: 0x65},
            udp_port: 61790, 
        };
        
        {
            let out = ser(&packet);
            assert_eq!(out, buf);
        }

        {
            let packet_2 = deser(&buf);
            assert_eq!(packet, packet_2);
        }
    }

    #[test]
    fn packet_ka() {
        let buf = vec![
            // UC01
            0x55,0x43,0x00,0x01,
            // size - 6 bytes
            0x06,0x00,
            // 4b(K), 41(A)
            0x4b,0x41,
            // 6b00 -> 6600 (k->f)
            0x6b,0x00,0x66,0x00
        ];

        let packet = UcPacket::KA(AddressPair { a: 0x6b, b: 0x66 });
        
        {
            let out = ser(&packet);
            assert_eq!(out, buf);
        }

        {
            let packet_2 = deser(&buf);
            assert_eq!(packet, packet_2);
        }
    }

    #[test]
    fn packet_pv() {
        // mute packet
        let buf = vec![
            // UC01
            0x55,0x43,0x00,0x01,
            // size - 26 bytes
            0x1a,0x00,
            // PV
            0x50,0x56,
            // 6b00 -> 6600 (k->f) // the response sends (f->k)
            0x6b,0x00,0x66,0x00,
            // string: line/ch1/mute
            // with zeros at the end
            0x6c,0x69,0x6e,0x65,0x2f,0x63,0x68,0x31,0x2f,0x6d,0x75,0x74,0x65,0x00,0x00,0x00,0x00,0x00,
            // this isn't always here - sometimes it's 00 00
            0x80,0x3f
        ];

        let packet = UcPacket::PV(
            AddressPair { a: 0x6b, b: 0x66 },
            "line/ch1/mute".to_string(),
            1.0
        );
        
        {
            let out = ser(&packet); 
            assert_eq!(out, buf);
        }

        {
            let packet_2 = deser(&buf);
            assert_eq!(packet, packet_2);
        }
    }

    #[test]
    fn packet_jm() {
        // mute packet
        let buf = vec![
            // UC01
            0x55,0x43,0x00,0x01,
            0x4f,0x00,
            // JM
            0x4a,0x4d,
            0x66,0x00,0x67,0x00,
            // json size
            0x45,0x00,0x00,0x00,
            // json data
            0x7b,0x22,0x69,0x64,0x22,0x3a,0x20,0x22,0x49,0x6e
            ,0x76,0x6f,0x6b,0x65,0x4d,0x65,0x74,0x68,0x6f,0x64,0x22,0x2c,0x22,0x75,0x72,0x6c
            ,0x22,0x3a,0x20,0x22,0x22,0x2c,0x22,0x6d,0x65,0x74,0x68,0x6f,0x64,0x22,0x3a,0x20
            ,0x22,0x67,0x65,0x74,0x4f,0x70,0x74,0x69,0x6f,0x6e,0x73,0x22,0x2c,0x22,0x63,0x61
            ,0x6c,0x6c,0x69,0x64,0x22,0x3a,0x20,0x31,0x30,0x30,0x7d,
        ];

        let json = r#"{"id": "InvokeMethod","url": "","method": "getOptions","callid": 100}"#;

        let packet = UcPacket::JM(
            AddressPair { a: 0x66, b: 0x67 },
            json.to_string(),
        );
        
        {
            let out = ser(&packet); 
            assert_eq!(out, buf);
        }

        {
            let packet_2 = deser(&buf);
            assert_eq!(packet, packet_2);
        }
    }
}
