use super::*;
use super::super::packets::*;
use super::super::server;
use mio;

pub struct EchoHandler {
    ip: mio::IpAddr,
}

impl EchoHandler {
    fn new(ip: mio::IpAddr) {
        EchoHandler{ip: ip};
    }
}

impl PacketHandler for EchoHandler {
    //This is the simplest handler of all of them.
    fn handle_incoming_packet(&mut self, packet: &Packet, server: &mut server::Server)->bool {
        match *packet {
            Packet::Echo(id) => {
                server.send(packet, &self.ip);
                true
            },
            _ => false
        }
    }
}
