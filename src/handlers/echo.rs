use super::*;
use super::super::packets::*;
use super::super::server::{self, Server};
use super::super::test_server;
use std::net;

pub struct EchoHandler {
    ip: net::IpAddr,
}

impl EchoHandler {
    fn new(ip: net::IpAddr)->EchoHandler {
        EchoHandler{ip: ip}
    }
}

impl PacketHandler for EchoHandler {
    //This is the simplest handler of all of them.
    fn handle_incoming_packet<T: Server>(&mut self, packet: &Packet, server: &mut T)->bool {
        match *packet {
            Packet::Echo(id) => {
                server.send(packet, &self.ip);
                true
            },
            _ => false
        }
    }
}

handler_test!(test_echo_handler, |server: &mut test_server::TestServer, ip: net::IpAddr| {
    let mut handler = EchoHandler::new(ip);
    handler.handle_incoming_packet(&Packet::Echo(16), server);
}, Packet::Echo(16));
