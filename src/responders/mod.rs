#![macro_use]

use super::packets;
use super::server;
use super::test_server;
use std::net;

//This macro generates a test, passing the second argument a server and ip.
//The macro then checks to see if we sent all the packets after the block.
//Per the Rust IRC this has to be before the mods.
macro_rules! responder_test {
    ($name: ident, $test: expr, $($expected: expr),*) => {
        #[test]
        fn $name() {
            let mut server = test_server::TestServer::new();
            let ip = net::IpAddr::V4(net::Ipv4Addr::new(127, 0, 0, 1));
            $test(&mut server, ip);
            let mut i = server.sent_packets.iter();
            $(assert_eq!(&(ip, $expected), i.next().unwrap());)*
        }
    }
}

mod connection;
mod echo;
mod heartbeat;
mod status;

//We have significanty less tests than the packets module.
//Consequently, they're in with the types they test.

pub use self::connection::*;
pub use self::echo::*;
pub use self::heartbeat::*;
pub use self::status::*;

pub trait PacketResponder {
    //Return true if and only if this handler handles the packet.
    //This function is called if and only if the packet is destined for an already-established connection and this handler is associated with it.
    //The server handles the initial association when connections are created.
    fn handle_incoming_packet<T: server::Server>(&mut self, packet: &packets::Packet, server: &mut T)->bool {
        false
    }
    //This variant is called when the packet is not for a connection.
    fn handle_incoming_packet_connectionless<T: server::Server>(&mut self, packet: &packets::Packet, ip: net::IpAddr, server: &mut T)->bool {
        false
    }
    //This variant is called in both cases, and is used primarily for the status responses.
    //It is called before either handle_incoming_packet or handle_incoming_packet_connectionless.
    fn handle_incoming_packet_always<T: server::Server>(&mut self, packet: &packets::Packet, ip: net::IpAddr, server: &mut T)->bool {
        false
    }
    //This happens if and only if get_tick_frequency returns a time.
    fn tick<T: server::Server>(&mut self, server: &mut T) {
    }
    //Return a time in MS.
    fn get_tick_frequency(&self)->Option<u32> {
        None
    }
}
