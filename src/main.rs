use std::net::UdpSocket;

use lab::build_query;

fn main() {
    let socket = UdpSocket::bind("0.0.0.0:0").expect("couldn't bind to address");

    const DOMAIN_NAME: &[u8; 15] = b"www.example.com";
    let query = build_query(*DOMAIN_NAME, 1);
    socket
        .send_to(&query, "8.8.8.8:53")
        .expect("couldn't send message");

    let mut buf = [0; 1024];
    socket.recv(&mut buf).expect("couldn't receive message");
}
