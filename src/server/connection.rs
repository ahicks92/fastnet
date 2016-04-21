use super::*;
use super::super::packets::{self, Packet};
use std::net;

#[derive(Debug)]
pub struct Connection {
    pub id: u64,
    pub address: net::SocketAddr,
}

impl Connection {
    pub fn new(id: u64, address: net::SocketAddr)->Connection {
        Connection {
            id: id,
            address: address,
        }
    }

    pub fn handle_incoming_packet<T: PacketSender>(&mut self, packet: &packets::Packet, sender: &mut T)->bool {
        match *packet {
            Packet::Echo(id) => {
                sender.send(packet, self.address);
                true
            },
            Packet::Heartbeat{counter: c, sent: s, received: r} => {
                true
            },
            _ => false
        }
    }
}
