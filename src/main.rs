use std::{io::Cursor, net::UdpSocket};

use lab::{build_query, parse_header, parse_question, parse_record};

fn main() {
    let socket = UdpSocket::bind("0.0.0.0:0").expect("couldn't bind to address");

    const DOMAIN_NAME: &[u8; 15] = b"www.example.com";
    let query = build_query(*DOMAIN_NAME, 1);
    socket
        .send_to(&query, "8.8.8.8:53")
        .expect("couldn't send message");

    let mut buf = [0; 1024];

    socket.recv(&mut buf).expect("couldn't receive message");

    let mut buf_curosr = Cursor::new(buf);
    dbg!(parse_header(&mut buf_curosr));
    dbg!(parse_question(&mut buf_curosr));
    dbg!(parse_record(&mut buf_curosr));
}
