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

        UcPacket::UM(buf) => {
            let size: u16 = (buf.len() as u16 + 2).try_into().unwrap();
            w.write(&size.to_le_bytes())?;
            w.write(&[b'U', b'M'])?;
            w.write(buf)?;
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

            // padding with 5 zeros
            w.write(&[0u8,0,0,0,0])?;

            // 2 bytes at the end
            if *val {
                w.write(&[0x80, 0x3f])?;
            } else {
                w.write(&[0x00, 0x00])?;
            }
        }

        UcPacket::FR(ap, some_number, buf) => {
            let size: u16 = ((buf.len() + 8) as u16).try_into().unwrap();
            w.write(&size.to_le_bytes())?;
            w.write(&[b'F', b'R'])?;
            write_address_pair(ap, w)?;
            w.write(&some_number.to_le_bytes())?;
            w.write(buf.as_bytes())?;
        }

        UcPacket::ZM(ap, buf) => {
            let size: u16 = ((buf.len() + 6) as u16).try_into().unwrap();
            w.write(&size.to_le_bytes())?;
            w.write(&[b'Z', b'M'])?;
            write_address_pair(ap, w)?;
            w.write(buf)?;
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
    match (bytes[0], bytes[1]) {
        (b'J',b'M') => {
            let address_pair = AddressPair {
                a: (bytes[2] as u16) | ((bytes[3] as u16) << 8),
                b: (bytes[4] as u16) | ((bytes[5] as u16) << 8),
            };
            // ignore 32-bit size
            Ok(UcPacket::JM(address_pair, str::from_utf8(&bytes[10..]).unwrap().to_string()))
        }
        (b'P',b'V') => {
            let address_pair = AddressPair {
                a: (bytes[2] as u16) | ((bytes[3] as u16) << 8),
                b: (bytes[4] as u16) | ((bytes[5] as u16) << 8),
            };
            
            let data_len = bytes.len() - 7;
            // 803f or 0000
            let val = bytes[bytes.len()-1] == 0x3f;

            Ok(UcPacket::PV(
                address_pair,
                str::from_utf8(&bytes[6..data_len]).unwrap().to_string(),
                val
            ))
        }
        (b'U',b'M') => {
            let payload: [u8; 6] = bytes[2..8].try_into().expect("Not enough bytes for UM packet");
            Ok(UcPacket::UM(payload))
        }
        (b'F',b'R') => {
            let address_pair = AddressPair {
                a: (bytes[2] as u16) | ((bytes[3] as u16) << 8),
                b: (bytes[4] as u16) | ((bytes[5] as u16) << 8),
            };
            let some_number = (bytes[4] as u16) + (bytes[5] as u16) << 8;
            let string_query = str::from_utf8(&bytes[6..]).unwrap().to_string();
            Ok(UcPacket::FR(address_pair, some_number, string_query))
        }
        (b'K',b'A') => {
            let address_pair = AddressPair {
                a: (bytes[2] as u16) | ((bytes[3] as u16) << 8),
                b: (bytes[4] as u16) | ((bytes[5] as u16) << 8),
            };
            Ok(UcPacket::KA(address_pair))
        }
        (b'Z',b'M') => {
            let address_pair = AddressPair {
                a: (bytes[2] as u16) | ((bytes[3] as u16) << 8),
                b: (bytes[4] as u16) | ((bytes[5] as u16) << 8),
            };

            Ok(UcPacket::ZM(address_pair, bytes[6..].to_vec()))
        }
        _ => {
            println!("Unknown packet encountered - {}{}", bytes[0], bytes[1]);
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

        let packet = UcPacket::UM([0x00,0x00,0x65,0x00,0x5e,0xf1]);
        
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
            true
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
