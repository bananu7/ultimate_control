use std::io::{Error, ErrorKind};

use tokio_util::codec::{Decoder, Encoder};
use bytes::{BytesMut, Buf};

use crate::types::*;
use crate::packet::{parse_packet_contents, ser};

pub struct PacketCodec {}

impl Decoder for PacketCodec {
    type Item = UcPacket;
    type Error = std::io::Error;

    fn decode(
        &mut self,
        src: &mut BytesMut
    ) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() < 6 {
            // Not enough data to read length marker.
            return Ok(None);
        }

        // verify the header
        if  src[0] != b'U' || // 'U'
            src[1] != b'C' || // 'C'
            src[2] != 0u8  ||
            src[3] != 1u8
        {
            return Err(Error::new(ErrorKind::Other, "header magic bytes invalid"));
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

        // Convert the data to a packet; header already stripped
        match parse_packet_contents(&data) {
            Ok(p) => Ok(Some(p)),
            Err(parse_error) => {
                Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    parse_error,
                ))
            },
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