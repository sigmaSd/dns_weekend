//In network packets, integers are always encoded in a big endian way
//(though little endian is the default in most other situations).

use std::mem::size_of;

#[derive(Default, Debug)]
pub struct DNSHeader {
    id: u16,
    flags: u16,
    num_questions: u16,
    num_answers: u16,
    num_authorities: u16,
    num_additionals: u16,
}
impl DNSHeader {
    fn to_bytes(&self) -> [u8; size_of::<DNSHeader>()] {
        [
            self.id.to_be_bytes(),
            self.flags.to_be_bytes(),
            self.num_questions.to_be_bytes(),
            self.num_answers.to_be_bytes(),
            self.num_authorities.to_be_bytes(),
            self.num_additionals.to_be_bytes(),
        ]
        .concat()
        .try_into()
        .unwrap()
    }
}

pub struct DNSQuestion<const N: usize> {
    name: [u8; N],
    type_: u16,
    class: u16,
}
impl<const N: usize> DNSQuestion<N> {
    /// R = N + 2
    fn to_bytes(&self) -> [u8; N + 4] {
        let mut result = [0u8; N + 4];
        result[..N].copy_from_slice(&self.name);
        result[N..N + 2].copy_from_slice(&self.type_.to_be_bytes());
        result[N + 2..N + 4].copy_from_slice(&self.class.to_be_bytes());

        result
    }
}

/// ASSUMPTION:
/// The length of each part fits in one byte
/// using this , we can conclude that the output len is the same as input len + 2 (TODO MATH)
fn encode_dns_name<const D: usize>(domain_name: [u8; D]) -> [u8; D + 2] {
    let mut encoded = [0u8; D + 2];
    let mut offset = 0;
    for part in domain_name.split(|byte| byte == &b'.') {
        let len_bytes: [u8; 1] = (u8::try_from(part.len())
            .expect("each dns name part must fit in one byte"))
        .to_be_bytes();
        encoded[offset..offset + len_bytes.len()].copy_from_slice(&len_bytes);
        encoded[offset + len_bytes.len()..offset + len_bytes.len() + part.len()]
            .copy_from_slice(&part);
        offset += len_bytes.len() + part.len();
    }
    encoded[offset..].copy_from_slice(b"\x00"); // not needed but just to be explicit
    encoded
}

const TYPE_A: u16 = 1;
const CLASS_IN: u16 = 1;

pub fn build_query<const D: usize>(
    domain_name: [u8; D],
    record_type: u16,
) -> [u8; size_of::<DNSHeader>() + D + 2 + 4]
where
    [(); D + 2]: Sized,
    [(); D + 2 + 4]: Sized,
{
    let name = encode_dns_name::<D>(domain_name);
    let id = 1; //NOTE random
    const RECURSION_DESIRED: u16 = 1 << 8;
    let header = DNSHeader {
        id,
        num_questions: 1,
        flags: RECURSION_DESIRED,
        ..Default::default()
    };
    let question = DNSQuestion {
        name,
        type_: record_type,
        class: CLASS_IN,
    };

    let header_bytes = header.to_bytes();
    let question_bytes = question.to_bytes();
    let mut result = [0u8; /*header*/ size_of::<DNSHeader>() + /*question*/ D + 2 + 4];
    result[..header_bytes.len()].copy_from_slice(&header_bytes);
    result[header_bytes.len()..].copy_from_slice(&question_bytes);
    result
}

#[test]
fn build_query_test() {
    const DOMAIN_NAME: &[u8; 11] = b"example.com";
    assert_eq!(
        &build_query(*DOMAIN_NAME, TYPE_A),
        b"\x00\x01\x01\x00\x00\x01\x00\x00\x00\x00\x00\x00\x07example\x03com\x00\x00\x01\x00\x01"
    );
}

#[test]
fn encode_dns_name_test() {
    const DOMAIN_NAME: &[u8; 10] = b"google.com";
    assert_eq!(&encode_dns_name(*DOMAIN_NAME), b"\x06google\x03com\x00")
}

#[test]
fn pack() {
    /*
    IN:  struct.pack('!HH', 5, 23)
    OUT: b'\x00\x05\x00\x17'
    */
    let mut result = [0u8; 4];
    result[..2].copy_from_slice(&5u16.to_be_bytes());
    result[2..].copy_from_slice(&23u16.to_be_bytes());
    assert_eq!(&result, b"\x00\x05\x00\x17");
}
#[test]
fn pack_format() {
    /*
     IN:  header_to_bytes(DNSHeader(id=0x1314, flags=0, num_questions=1, num_additionals=0, num_authorities=0, num_answers=0))
     OUT: b'\x13\x14\x00\x00\x00\x01\x00\x00\x00\x00\x00\x00'
    */
    let header = DNSHeader {
        id: 0x1314,
        flags: 0,
        num_questions: 1,
        num_additionals: 0,
        num_authorities: 0,
        num_answers: 0,
    };
    assert_eq!(
        &header.to_bytes(),
        b"\x13\x14\x00\x00\x00\x01\x00\x00\x00\x00\x00\x00"
    );
}
