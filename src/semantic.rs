use miniz_oxide::inflate::decompress_to_vec_with_limit;

use crate::types::{UcPacket};

pub fn print_packet(packet: &UcPacket) {
    match packet {
        UcPacket::ZM{ap, unknown, compressed_payload} => {
            println!("<- ZM ({:02X?}, {}, {:02X?})", ap, unknown, decode_zm_packet_data(compressed_payload));
        }
        _ => {
            println!("<- {:02X?}", packet);
        }
    }
}

fn decode_zm_packet_data(data: &Vec<u8>) -> String {
    let payload = decompress_to_vec_with_limit(&data[2..], 600000).unwrap();
    String::from_utf8(payload).unwrap()
}
