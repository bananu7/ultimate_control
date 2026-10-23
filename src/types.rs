#[derive(Debug, PartialEq, Clone)]
pub struct AddressPair {
    pub a: u16,
    pub b: u16,
}

#[derive(Debug, PartialEq, Clone)]
pub enum UcPacket {
    JM(AddressPair, String),
    UM {
        ap: AddressPair,
        udp_port: u16,
    },
    KA(AddressPair),
    PV(AddressPair, String, f32),
    FR(AddressPair, u16, String),

    ZM {
        ap: AddressPair,
        unknown: u32,
        compressed_payload: Vec<u8>,
    },

    PS(AddressPair, Vec<u8>),
    PL(AddressPair, String, Vec<String>),

    /// A well-framed packet whose two-letter type we do not recognise yet
    /// (the device also speaks CK, MS, PC, FD, ... which are not decoded).
    /// Kept as raw bytes so the stream stays alive and the payload can be
    /// inspected.
    Unknown {
        tag: [u8; 2],
        ap: AddressPair,
        data: Vec<u8>,
    },
}

impl UcPacket {
    /// Two-letter wire type of this packet.
    pub fn tag(&self) -> [u8; 2] {
        match self {
            UcPacket::JM(..) => *b"JM",
            UcPacket::UM { .. } => *b"UM",
            UcPacket::KA(..) => *b"KA",
            UcPacket::PV(..) => *b"PV",
            UcPacket::FR(..) => *b"FR",
            UcPacket::ZM { .. } => *b"ZM",
            UcPacket::PS(..) => *b"PS",
            UcPacket::PL(..) => *b"PL",
            UcPacket::Unknown { tag, .. } => *tag,
        }
    }
}
