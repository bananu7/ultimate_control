use std::io::prelude::*;
use std::io::{Error, ErrorKind};
use std::str;
//use byteorder::{ByteOrder, LittleEndian}; 

#[derive(Debug)]
#[derive(PartialEq)]
struct AddressPair {
    a: u16,
    b: u16,
}

#[derive(Debug)]
#[derive(PartialEq)]
enum UcPacket {
    JM(AddressPair, String),
    UM([u8; 6]),
    KA(AddressPair),
    PV(AddressPair, String),
    FR(AddressPair, u16, String)
}

fn write_address_pair<Writer: Write>(ap: &AddressPair, w: &mut Writer) -> std::io::Result<()> {
    w.write(&ap.a.to_le_bytes())?;
    w.write(&ap.b.to_le_bytes())?;
    Ok(())
}

fn write_packet<Writer: Write>(p: &UcPacket, w: &mut Writer) -> std::io::Result<()>  {
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

        UcPacket::PV(ap, buf) => {
            let size: u16 = ((buf.len() + 4) as u16).try_into().unwrap();
            w.write(&size.to_le_bytes())?;
            w.write(&[b'P', b'V'])?;
            write_address_pair(ap, w)?;
            w.write(buf.as_bytes())?;
        }

        UcPacket::FR(ap, some_number, buf) => {
            let size: u16 = ((buf.len() + 6) as u16).try_into().unwrap();
            w.write(&size.to_le_bytes())?;
            w.write(&[b'F', b'R'])?;
            write_address_pair(ap, w)?;
            w.write(&some_number.to_le_bytes())?;
            w.write(buf.as_bytes())?;
        }
    }

    Ok(())
}

fn read_packet<Reader: Read>(stream: &mut Reader) -> std::io::Result<UcPacket> {
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

    let size: u16 = (header[4] as u16) + (header[5] as u16) << 8;

    let mut buf = Vec::new();
    stream.take(size as u64).read_to_end(&mut buf)?;

    println!("{:?}", buf);

    parse_packet_contents(&buf)
}

fn parse_packet_contents(bytes: &Vec<u8>) -> std::io::Result<UcPacket> {
    println!("{}{}", bytes[0], bytes[1]);
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
            Ok(UcPacket::PV(address_pair, str::from_utf8(&bytes[6..]).unwrap().to_string()))
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
        _ => Err(Error::new(ErrorKind::Other, "Invalid packet"))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::io::{Cursor, Read, Seek, SeekFrom};

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
            let mut c = Cursor::new(Vec::new());
            write_packet(&packet, &mut c).unwrap();
            
            let mut out = Vec::new();
            c.seek(SeekFrom::Start(0)).unwrap();
            c.read_to_end(&mut out).unwrap();

            assert_eq!(out, buf);
        }

        {
            let mut stream = Cursor::new(&buf);
            let packet_2 = read_packet(&mut stream).unwrap();
            assert_eq!(packet, packet_2);
        }
    }

}