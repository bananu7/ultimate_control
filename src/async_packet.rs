use std::io::Error;

use bytes::{Buf, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

use crate::packet::{parse_packet_contents, ser};
use crate::types::*;

pub struct PacketCodec {}

/// Best-effort split of a frame body into (tag, address pair, payload),
/// tolerating frames shorter than the 6-byte minimum.
fn split_raw(data: &[u8]) -> ([u8; 2], AddressPair, Vec<u8>) {
    let byte = |i: usize| data.get(i).copied().unwrap_or(0);
    let tag = [byte(0), byte(1)];
    let ap = AddressPair {
        a: u16::from_le_bytes([byte(2), byte(3)]),
        b: u16::from_le_bytes([byte(4), byte(5)]),
    };
    let payload = data.get(6..).unwrap_or(&[]).to_vec();
    (tag, ap, payload)
}

impl Decoder for PacketCodec {
    type Item = UcPacket;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() < 6 {
            // Not enough data to read length marker.
            return Ok(None);
        }

        // verify the header
        if src[0] != b'U' || // 'U'
            src[1] != b'C' || // 'C'
            src[2] != 0u8  ||
            src[3] != 1u8
        {
            return Err(Error::other("header magic bytes invalid"));
        }

        // Read length marker.
        let mut length_bytes = [0u8; 2];
        length_bytes.copy_from_slice(&src[4..6]);
        let length = u16::from_le_bytes(length_bytes) as usize;

        if src.len() < 6 + length {
            // The full string has not yet arrived.
            //
            // We reserve more space in the buffer. This is not strictly
            // necessary, but is a good idea performance-wise.
            src.reserve(6 + length - src.len());

            // We inform the Framed that we need more bytes to form the next
            // frame.
            return Ok(None);
        }

        // Use advance to modify src such that it no longer contains
        // this frame.
        let data = src[6..6 + length].to_vec();
        src.advance(6 + length);

        // Convert the data to a packet; header already stripped.
        //
        // A decode error makes `Framed` end the stream for good, which is far
        // worse than skipping one bad frame, so a frame that is well delimited
        // but does not parse is surfaced as `Unknown` instead of an error.
        match parse_packet_contents(&data) {
            Ok(p) => Ok(Some(p)),
            Err(parse_error) => {
                eprintln!(
                    "skipping unparseable {:?} frame ({} bytes): {}",
                    String::from_utf8_lossy(&data[..data.len().min(2)]),
                    data.len(),
                    parse_error
                );
                let (tag, ap, payload) = split_raw(&data);
                Ok(Some(UcPacket::Unknown {
                    tag,
                    ap,
                    data: payload,
                }))
            }
        }
    }
}

impl Encoder<UcPacket> for PacketCodec {
    type Error = std::io::Error;

    fn encode(&mut self, packet: UcPacket, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let buf = ser(&packet);
        dst.reserve(buf.len());
        dst.extend_from_slice(&buf);
        Ok(())
    }
}
