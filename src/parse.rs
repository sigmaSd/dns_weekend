use std::io::Read;

use crate::query::DNSHeader;

struct DNSRecord<const N: usize, const D: usize> {
    name: [u8; N],
    type_: u16,
    class: u16,
    ttl: u32,
    data: [u8; D],
}

pub fn parse_header(reader: &mut impl Read) -> DNSHeader {
    let mut header = [0; 12];
    reader
        .read_exact(&mut header)
        .expect("couldn't read header");
    unsafe { std::mem::transmute::<[u8; 12], DNSHeader>(header) }
}

pub fn decode_name_simple(reader: &mut impl Read) -> String {
    let mut parts = vec![];
    let mut len = [0; 1];
    while reader.read_exact(&mut len).is_ok() {
        if len[0] == 0 {
            break;
        }
        let mut part = vec![0; len[0] as usize];
        reader.read_exact(&mut part).expect("couldn't read part");
        parts.push(String::from_utf8(part).unwrap());
    }
    parts.join(".")
}

pub fn parse_question(reader: &mut impl Read) -> DNSQuestion {
    let name = decode_name_simple(reader);
    let mut data = [0; 4];
    reader.read_exact(&mut data).expect("couldn't read data");
    let type_ = u16::from_be_bytes(data[0..2].try_into().unwrap());
    let class = u16::from_be_bytes(data[2..4].try_into().unwrap());

    DNSQuestion { name, type_, class }
}

#[derive(Debug)]
pub struct DNSQuestion {
    name: String,
    type_: u16,
    class: u16,
}
