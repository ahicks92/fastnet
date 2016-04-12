use super::*;
use super::super::packets::*;
use super::super::server::*;
use std::net;

#[derive(Debug, Default)]
pub struct EchoResponder;

impl EchoResponder {
    pub fn new()->EchoResponder {
        EchoResponder::default()
    }
}

impl ConnectedPacketResponder for EchoResponder {
    fn handle_incoming_packet<T: Server>(&mut self, packet: &Packet, connection: &Connection, server: &mut T)->bool {
        match *packet {
            Packet::Echo(id) => {
                server.send(packet, connection.ip, connection.port);
                true
            },
            _ => false
        }
    }
}

responder_test!(test_echo_responder, |server: &mut TestServer, connection: &Connection, ip: net::IpAddr| {
    let mut responder = EchoResponder::new();
    responder.handle_incoming_packet(&Packet::Echo(16), connection, server);
}, Packet::Echo(16));
