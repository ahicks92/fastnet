#![macro_use]

use super::packets;
use super::server::*;
use std::net;

//This macro generates a test, passing the second argument a server and ip.
//The macro then checks to see if we sent all the packets after the block.
//Per the Rust IRC this has to be before the mods.
macro_rules! responder_test {
    ($name: ident, $test: expr, $($expected: expr),*) => {
        #[test]
        fn $name() {
            let mut server = TestServer::new();
            let ip = net::IpAddr::V4(net::Ipv4Addr::new(127, 0, 0, 1));
            let address = net::SocketAddr::new(ip, 0);
            let connection = Connection::new(1, address);
            $test(&mut server, &connection, address);
            let mut c = 0usize;
            let mut i = server.sent_packets.iter();
            $(assert_eq!($expected, i.next().unwrap().1); c+=1;)*
            assert_eq!(c, server.sent_packets.len());
        }
    }
}

mod echo;
mod heartbeat;
mod status;

pub use self::echo::*;
pub use self::heartbeat::*;
pub use self::status::*;
