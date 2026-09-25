use miniz_oxide::inflate::decompress_to_vec_with_limit;

use crate::types::UcPacket;

pub fn print_packet(packet: &UcPacket, outgoing: bool) {
    if outgoing {
        print!("-> ");
    } else {
        print!("<- ");
    }

    match packet {
        UcPacket::ZM {
            ap,
            unknown,
            compressed_payload,
        } => {
            println!(
                "ZM ({:02X?}, {}, {})",
                ap,
                unknown,
                decode_zm_packet_data(compressed_payload)
            );
        }

        UcPacket::UM { ap, udp_port } => {
            println!("UM ({:02X?}, {})", ap, udp_port);
        }

        _ => {
            println!("{:02X?}", packet);
        }
    }
}

/// Inflate the zlib-compressed ZM payload (first two bytes are a zlib header).
/// Returns a diagnostic string instead of panicking on malformed data coming
/// from the network.
fn decode_zm_packet_data(data: &[u8]) -> String {
    let Some(deflated) = data.get(2..) else {
        return format!("<ZM payload too short: {:02X?}>", data);
    };

    match decompress_to_vec_with_limit(deflated, 600_000) {
        Ok(payload) => String::from_utf8_lossy(&payload).into_owned(),
        Err(e) => format!("<ZM payload failed to inflate: {:?}>", e),
    }
}
