use std::io::Cursor;
use std::io::Error;
use std::io::prelude::*;
use std::str;

use crate::types::*;

const HEADER: &[u8; 4] = b"UC\x00\x01";

// those are convenience functions that operate directly on memory
pub fn ser(packet: &UcPacket) -> Vec<u8> {
    let mut out = Vec::new();
    write_packet(packet, &mut out).unwrap();
    out
}

pub fn deser(buf: &[u8]) -> UcPacket {
    let mut stream = Cursor::new(buf);
    read_packet(&mut stream).unwrap()
}

fn size_u16(len: usize) -> std::io::Result<u16> {
    u16::try_from(len).map_err(|_| Error::other("packet payload too large"))
}

// Those functions operate on synchronous Read/Write objects
fn write_address_pair<Writer: Write>(ap: &AddressPair, w: &mut Writer) -> std::io::Result<()> {
    w.write_all(&ap.a.to_le_bytes())?;
    w.write_all(&ap.b.to_le_bytes())?;
    Ok(())
}

pub fn write_packet<Writer: Write>(p: &UcPacket, w: &mut Writer) -> std::io::Result<()> {
    w.write_all(HEADER)?;

    match p {
        UcPacket::JM(ap, s) => {
            let size = size_u16(s.len() + 10)?; // 10 bytes for extra metadata
            w.write_all(&size.to_le_bytes())?;
            w.write_all(b"JM")?;
            write_address_pair(ap, w)?;
            w.write_all(&(s.len() as u32).to_le_bytes())?;
            w.write_all(s.as_bytes())?;
        }

        UcPacket::UM { ap, udp_port } => {
            let size: u16 = 8;
            w.write_all(&size.to_le_bytes())?;
            w.write_all(b"UM")?;
            write_address_pair(ap, w)?;
            w.write_all(&udp_port.to_le_bytes())?;
        }

        UcPacket::KA(ap) => {
            let size: u16 = 6;
            w.write_all(&size.to_le_bytes())?;
            w.write_all(b"KA")?;
            write_address_pair(ap, w)?;
        }

        UcPacket::PV(ap, name, val) => {
            // tag (2) + address pair (4) + name + 3 zero bytes + f32 (4)
            let size = size_u16(name.len() + 6 + 3 + 4)?;
            w.write_all(&size.to_le_bytes())?;
            w.write_all(b"PV")?;
            write_address_pair(ap, w)?;

            // parameter name
            w.write_all(name.as_bytes())?;

            // padding with 3 zeros
            w.write_all(&[0u8, 0, 0])?;

            // float
            w.write_all(&val.to_le_bytes())?;
        }

        UcPacket::FR(ap, some_number, buf) => {
            let size = size_u16(buf.len() + 8)?;
            w.write_all(&size.to_le_bytes())?;
            w.write_all(b"FR")?;
            write_address_pair(ap, w)?;
            w.write_all(&some_number.to_le_bytes())?;
            w.write_all(buf.as_bytes())?;
        }

        UcPacket::ZM {
            ap,
            unknown,
            compressed_payload,
        } => {
            let size = size_u16(compressed_payload.len() + 10)?;
            w.write_all(&size.to_le_bytes())?;
            w.write_all(b"ZM")?;
            write_address_pair(ap, w)?;
            w.write_all(&unknown.to_le_bytes())?;
            w.write_all(compressed_payload)?;
        }

        UcPacket::PS(ap, buf) => {
            let size = size_u16(buf.len() + 6)?;
            w.write_all(&size.to_le_bytes())?;
            w.write_all(b"PS")?;
            write_address_pair(ap, w)?;
            w.write_all(buf)?;
        }

        UcPacket::PL(ap, name, names) => {
            let size_of_names: usize = names.iter().map(|n| n.len()).sum::<usize>() + // each of the names
                names.len(); // newlines between names and 0 at the end
            let size = size_u16(6 + name.len() + 7 + size_of_names)?;
            w.write_all(&size.to_le_bytes())?;
            w.write_all(b"PL")?;
            write_address_pair(ap, w)?;
            w.write_all(name.as_bytes())?;
            w.write_all(&[0u8; 7])?;
            for (i, n) in names.iter().enumerate() {
                if i > 0 {
                    w.write_all(b"\n")?;
                }
                w.write_all(n.as_bytes())?;
            }
            w.write_all(&[0u8])?;
        }
    }

    Ok(())
}

pub fn read_packet<Reader: Read>(stream: &mut Reader) -> std::io::Result<UcPacket> {
    let mut header = [0u8; 6];
    stream.read_exact(&mut header)?;

    if header[0..4] != HEADER[..] {
        return Err(Error::other("header magic bytes invalid"));
    }

    let size = u16::from_le_bytes([header[4], header[5]]);

    let mut buf = Vec::with_capacity(size as usize);
    stream.take(size as u64).read_to_end(&mut buf)?;

    parse_packet_contents(&buf)
}

fn utf8(bytes: &[u8]) -> std::io::Result<String> {
    str::from_utf8(bytes)
        .map(str::to_string)
        .map_err(|e| Error::new(std::io::ErrorKind::InvalidData, e))
}

fn need(bytes: &[u8], len: usize, what: &str) -> std::io::Result<()> {
    if bytes.len() < len {
        return Err(Error::new(
            std::io::ErrorKind::UnexpectedEof,
            format!("{what} packet too short ({} < {len} bytes)", bytes.len()),
        ));
    }
    Ok(())
}

pub fn parse_packet_contents(bytes: &[u8]) -> std::io::Result<UcPacket> {
    need(bytes, 6, "UC")?;

    let address_pair = AddressPair {
        a: u16::from_le_bytes([bytes[2], bytes[3]]),
        b: u16::from_le_bytes([bytes[4], bytes[5]]),
    };

    match (bytes[0], bytes[1]) {
        (b'J', b'M') => {
            need(bytes, 10, "JM")?;
            // ignore 32-bit size
            Ok(UcPacket::JM(address_pair, utf8(&bytes[10..])?))
        }
        (b'P', b'V') => {
            // name + 3 zero bytes + 4-byte float
            need(bytes, 6 + 7, "PV")?;
            let data_len = bytes.len() - 7;
            // 803f or 0000
            let f = bytes.len() - 4;
            let val = f32::from_le_bytes([bytes[f], bytes[f + 1], bytes[f + 2], bytes[f + 3]]);

            Ok(UcPacket::PV(address_pair, utf8(&bytes[6..data_len])?, val))
        }
        (b'U', b'M') => {
            if bytes.len() != 8 {
                return Err(Error::other("Wrong amount of bytes for UM packet"));
            }
            let udp_port = u16::from_le_bytes([bytes[6], bytes[7]]);

            Ok(UcPacket::UM {
                ap: address_pair,
                udp_port,
            })
        }
        (b'F', b'R') => {
            need(bytes, 8, "FR")?;
            let some_number = u16::from_le_bytes([bytes[6], bytes[7]]);
            let string_query = utf8(&bytes[8..])?;
            Ok(UcPacket::FR(address_pair, some_number, string_query))
        }
        (b'K', b'A') => Ok(UcPacket::KA(address_pair)),
        (b'Z', b'M') => {
            need(bytes, 10, "ZM")?;
            Ok(UcPacket::ZM {
                ap: address_pair,
                unknown: u32::from_le_bytes([bytes[6], bytes[7], bytes[8], bytes[9]]),
                compressed_payload: bytes[10..].to_vec(),
            })
        }
        (b'P', b'S') => Ok(UcPacket::PS(address_pair, bytes[6..].to_vec())),
        (b'P', b'L') => {
            let rest = &bytes[6..];

            // key runs until the first of 7 zero bytes
            let key_end = rest
                .iter()
                .position(|&b| b == 0x00)
                .ok_or_else(|| Error::other("PL packet is missing the key terminator"))?;
            let key = utf8(&rest[..key_end])?;

            // items are separated by newlines and terminated by a 0 byte
            let list = rest.get(key_end + 7..).unwrap_or(&[]);
            let list_end = list.iter().position(|&b| b == 0x00).unwrap_or(list.len());
            let names = list[..list_end]
                .split(|&b| b == 0x0a)
                .map(utf8)
                .collect::<std::io::Result<Vec<String>>>()?;

            Ok(UcPacket::PL(address_pair, key, names))
        }
        _ => {
            println!(
                "Unknown packet encountered - {}{}",
                bytes[0] as char, bytes[1] as char
            );
            println!("{:02X?}", &bytes[2..]);
            Err(Error::other("Invalid packet"))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn roundtrip(packet: &UcPacket, buf: &[u8]) {
        assert_eq!(ser(packet), buf);
        assert_eq!(&deser(buf), packet);
    }

    #[test]
    fn packet_um() {
        let buf = vec![
            // UC01
            0x55, 0x43, 0x00, 0x01, // size
            0x08, 0x00, // UM
            0x55, 0x4d, // 6 bytes data
            0x00, 0x00, 0x65, 0x00, 0x5e, 0xf1,
        ];

        let packet = UcPacket::UM {
            ap: AddressPair { a: 0, b: 0x65 },
            udp_port: 61790,
        };

        roundtrip(&packet, &buf);
    }

    #[test]
    fn packet_ka() {
        let buf = vec![
            // UC01
            0x55, 0x43, 0x00, 0x01, // size - 6 bytes
            0x06, 0x00, // 4b(K), 41(A)
            0x4b, 0x41, // 6b00 -> 6600 (k->f)
            0x6b, 0x00, 0x66, 0x00,
        ];

        let packet = UcPacket::KA(AddressPair { a: 0x6b, b: 0x66 });

        roundtrip(&packet, &buf);
    }

    #[test]
    fn packet_pv() {
        // mute packet
        let buf = vec![
            // UC01
            0x55, 0x43, 0x00, 0x01, // size - 26 bytes
            0x1a, 0x00, // PV
            0x50, 0x56, // 6b00 -> 6600 (k->f) // the response sends (f->k)
            0x6b, 0x00, 0x66, 0x00,
            // string: line/ch1/mute
            // with zeros at the end
            0x6c, 0x69, 0x6e, 0x65, 0x2f, 0x63, 0x68, 0x31, 0x2f, 0x6d, 0x75, 0x74, 0x65, 0x00,
            0x00, 0x00, 0x00, 0x00, // this isn't always here - sometimes it's 00 00
            0x80, 0x3f,
        ];

        let packet = UcPacket::PV(
            AddressPair { a: 0x6b, b: 0x66 },
            "line/ch1/mute".to_string(),
            1.0,
        );

        roundtrip(&packet, &buf);
    }

    #[test]
    fn packet_jm() {
        let buf = vec![
            // UC01
            0x55, 0x43, 0x00, 0x01, 0x4f, 0x00, // JM
            0x4a, 0x4d, 0x66, 0x00, 0x67, 0x00, // json size
            0x45, 0x00, 0x00, 0x00, // json data
            0x7b, 0x22, 0x69, 0x64, 0x22, 0x3a, 0x20, 0x22, 0x49, 0x6e, 0x76, 0x6f, 0x6b, 0x65,
            0x4d, 0x65, 0x74, 0x68, 0x6f, 0x64, 0x22, 0x2c, 0x22, 0x75, 0x72, 0x6c, 0x22, 0x3a,
            0x20, 0x22, 0x22, 0x2c, 0x22, 0x6d, 0x65, 0x74, 0x68, 0x6f, 0x64, 0x22, 0x3a, 0x20,
            0x22, 0x67, 0x65, 0x74, 0x4f, 0x70, 0x74, 0x69, 0x6f, 0x6e, 0x73, 0x22, 0x2c, 0x22,
            0x63, 0x61, 0x6c, 0x6c, 0x69, 0x64, 0x22, 0x3a, 0x20, 0x31, 0x30, 0x30, 0x7d,
        ];

        let json = r#"{"id": "InvokeMethod","url": "","method": "getOptions","callid": 100}"#;

        let packet = UcPacket::JM(AddressPair { a: 0x66, b: 0x67 }, json.to_string());

        roundtrip(&packet, &buf);
    }

    #[test]
    fn packet_fr() {
        let mut buf = vec![
            // UC01
            0x55, 0x43, 0x00, 0x01, // size - 8 + 3 bytes
            0x0b, 0x00, // FR
            0x46, 0x52, // address pair
            0x6b, 0x00, 0x66, 0x00, // some_number = 0x0102 little endian
            0x02, 0x01,
        ];
        buf.extend_from_slice(b"abc");

        let packet = UcPacket::FR(AddressPair { a: 0x6b, b: 0x66 }, 0x0102, "abc".to_string());

        roundtrip(&packet, &buf);
    }

    #[test]
    fn packet_pl() {
        let mut buf = vec![
            // UC01
            0x55, 0x43, 0x00, 0x01, // size: 6 + 3 ("key") + 7 + (3 + 2 + 2) names
            0x18, 0x00, // PL
            0x50, 0x4c, // address pair
            0x6b, 0x00, 0x66, 0x00,
        ];
        buf.extend_from_slice(b"key");
        buf.extend_from_slice(&[0u8; 7]);
        buf.extend_from_slice(b"one\ntwo\0");

        let packet = UcPacket::PL(
            AddressPair { a: 0x6b, b: 0x66 },
            "key".to_string(),
            vec!["one".to_string(), "two".to_string()],
        );

        roundtrip(&packet, &buf);
    }

    #[test]
    fn truncated_packets_are_errors_not_panics() {
        assert!(parse_packet_contents(&[]).is_err());
        assert!(parse_packet_contents(b"PV\x6b\x00\x66\x00").is_err());
        assert!(parse_packet_contents(b"ZM\x6b\x00\x66\x00\x01").is_err());
        assert!(parse_packet_contents(b"JM\x6b\x00\x66\x00\xff\xff").is_err());
    }
}
