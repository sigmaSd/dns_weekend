use std::net::UdpSocket;

use lab::{build_query, ip_to_string, DNSPacket, TYPE_A};

fn main() {
    let socket = UdpSocket::bind("0.0.0.0:0").expect("couldn't bind to address");

    // can't use a function because of a bug in const gen expr
    macro_rules! lookup_domain {
        ($n: tt) => {
            let query = build_query(*$n, TYPE_A);
            socket
                .send_to(&query, "8.8.8.8:53")
                .expect("couldn't send message");

            let mut buf = [0; 1024];
            socket.recv(&mut buf).expect("couldn't receive message");

            let packet = DNSPacket::parse(buf);
            dbg!(&packet);
            dbg!(ip_to_string(packet.answers[0].data.clone()));
        };
    }

    lookup_domain!(b"example.com");
    lookup_domain!(b"recurse.com");
}
