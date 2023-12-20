use miniz_oxide::inflate::decompress_to_vec_with_limit;

use crate::types::{UcPacket};

pub fn print_packet(packet: &UcPacket) {
    match packet {
        UcPacket::ZM(address_pair, buf) => {
            println!("<- ZM ({:02X?}, {:02X?})", address_pair, decode_zm_packet_data(buf));
        }
        _ => {
            println!("<- {:02X?}", packet);
        }
    }
}

fn decode_zm_packet_data(data: &Vec<u8>) -> String {
    let payload = decompress_to_vec_with_limit(&data[6..], 600000).unwrap();
    String::from_utf8(payload).unwrap()
}
