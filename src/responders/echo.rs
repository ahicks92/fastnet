use super::*;
use super::super::packets::*;
use super::super::server::*;
use std::net;

#[derive(Debug)]
pub struct EchoResponder {
    ip: net::IpAddr,
}

impl EchoResponder {
    pub fn new(ip: net::IpAddr)->EchoResponder {
        EchoResponder{ip: ip}
    }
}

impl ConnectedPacketResponder for EchoResponder {
    fn handle_incoming_packet<T: Server>(&mut self, packet: &Packet, server: &mut T)->bool {
        match *packet {
            Packet::Echo(id) => {
                server.send(packet.clone(), self.ip);
                true
            },
            _ => false
        }
    }
}

responder_test!(test_echo_responder, |server: &mut TestServer, ip: net::IpAddr| {
    let mut responder = EchoResponder::new(ip);
    responder.handle_incoming_packet(&Packet::Echo(16), server);
}, Packet::Echo(16));
