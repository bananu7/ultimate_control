#[derive(Debug)]
#[derive(PartialEq)]
pub struct AddressPair {
    pub a: u16,
    pub b: u16,
}

#[derive(Debug)]
#[derive(PartialEq)]
pub enum UcPacket {
    JM(AddressPair, String),
    UM([u8; 6]),
    KA(AddressPair),
    PV(AddressPair, String, f32),
    FR(AddressPair, u16, String),

    ZM{ 
        ap: AddressPair,
        unknown: u32,
        compressed_payload: Vec<u8>,
    },

    PS(AddressPair, Vec<u8>),
    PL(AddressPair, String, Vec<String>),
}
