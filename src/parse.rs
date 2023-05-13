use std::io::{Read, Seek};

use crate::query::DNSHeader;
use std::io::Cursor;

#[derive(Debug)]
pub struct DNSRecord {
    pub name: String,
    pub type_: u16,
    pub class: u16,
    pub ttl: u32,
    pub data: Vec<u8>,
}

pub fn parse_header(reader: &mut Cursor<[u8; 1024]>) -> DNSHeader {
    let mut header = [0; 12];
    reader.read_exact(&mut header).unwrap();

    DNSHeader {
        id: u16::from_be_bytes([header[0], header[1]]),
        flags: u16::from_be_bytes([header[2], header[3]]),
        num_questions: u16::from_be_bytes([header[4], header[5]]),
        num_answers: u16::from_be_bytes([header[6], header[7]]),
        num_authorities: u16::from_be_bytes([header[8], header[9]]),
        num_additionals: u16::from_be_bytes([header[10], header[11]]),
    }
}

pub fn decode_name(reader: &mut Cursor<[u8; 1024]>) -> String {
    let mut parts = vec![];
    let mut len = [0; 1];
    while reader.read_exact(&mut len).is_ok() {
        if len[0] == 0 {
            break;
        }
        if len[0] & 0b1100_0000 != 0 {
            parts.push(decode_compressed_name(len[0], reader));
            break;
        }
        let mut part = vec![0; len[0] as usize];
        reader.read_exact(&mut part).expect("couldn't read part");
        parts.push(String::from_utf8(part).unwrap());
    }
    parts.join(".")
}

fn decode_compressed_name(len: u8, reader: &mut Cursor<[u8; 1024]>) -> String {
    let mut next_byte = [0; 1];
    reader
        .read_exact(&mut next_byte)
        .expect("couldn't read next byte");

    let mut pointer_bytes = vec![(len & 0b0011_1111)];
    pointer_bytes.extend_from_slice(&next_byte);

    let pointer = ((pointer_bytes[0] as u16) << 8) | (pointer_bytes[1] as u16);

    let current_pos = reader.position();
    reader
        .seek(std::io::SeekFrom::Start(pointer as u64))
        .unwrap();
    let result = decode_name(reader);
    reader.seek(std::io::SeekFrom::Start(current_pos)).unwrap();

    result
}

pub fn decode_name_simple(reader: &mut Cursor<[u8; 1024]>) -> String {
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

pub fn parse_question(reader: &mut Cursor<[u8; 1024]>) -> DNSQuestion {
    let name = decode_name(reader);
    let mut data = [0; 4];
    reader.read_exact(&mut data).expect("couldn't read data");
    let type_ = u16::from_be_bytes(data[0..2].try_into().unwrap());
    let class = u16::from_be_bytes(data[2..4].try_into().unwrap());

    DNSQuestion { name, type_, class }
}

pub fn parse_record(reader: &mut Cursor<[u8; 1024]>) -> DNSRecord {
    let name = decode_name(reader);

    let mut data = [0; 10];
    reader.read_exact(&mut data).expect("couldn't read data");
    let type_ = u16::from_be_bytes(data[0..2].try_into().unwrap());
    let class = u16::from_be_bytes(data[2..4].try_into().unwrap());
    let ttl = u32::from_be_bytes(data[4..8].try_into().unwrap());
    let data_len = u16::from_be_bytes(data[8..10].try_into().unwrap());

    let mut data = vec![0; data_len as usize];
    reader.read_exact(&mut data).expect("couldn't read data");

    DNSRecord {
        name,
        data,
        type_,
        class,
        ttl,
    }
}

#[derive(Debug)]
pub struct DNSQuestion {
    name: String,
    type_: u16,
    class: u16,
}

#[derive(Debug)]
pub struct DNSPacket {
    pub header: DNSHeader,
    pub questions: Vec<DNSQuestion>,
    pub answers: Vec<DNSRecord>,
    pub authorities: Vec<DNSRecord>,
    pub additionals: Vec<DNSRecord>,
}

impl DNSPacket {
    pub fn parse(data: [u8; 1024]) -> Self {
        let mut reader = Cursor::new(data);
        let header = parse_header(&mut reader);
        let questions = (0..header.num_questions)
            .map(|_| parse_question(&mut reader))
            .collect();
        let answers = (0..header.num_answers)
            .map(|_| parse_record(&mut reader))
            .collect();
        let authorities = (0..header.num_authorities)
            .map(|_| parse_record(&mut reader))
            .collect();
        let additionals = (0..header.num_additionals)
            .map(|_| parse_record(&mut reader))
            .collect();

        DNSPacket {
            header,
            questions,
            answers,
            authorities,
            additionals,
        }
    }
}

pub fn ip_to_string(ip: Vec<u8>) -> String {
    ip.into_iter()
        .map(|p| p.to_string())
        .collect::<Vec<_>>()
        .join(".")
}
