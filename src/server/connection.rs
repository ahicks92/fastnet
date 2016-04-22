use super::*;
use super::super::packets::{self, Packet};
use std::net;

#[derive(Debug, Copy, Clone)]
pub enum ConnectionState {
    Initialized,
    Establishing,
    Established,
    Closed,
}

#[derive(Debug)]
pub struct Connection {
    pub state: ConnectionState,
    pub local_id: u64,
    pub remote_id: u64,
    pub address: net::SocketAddr,
}

impl Connection {
    pub fn new(address: net::SocketAddr, local_id: u64)->Connection {
        Connection::with_remote_id(address, local_id, 0)
    }

    pub fn with_remote_id(address: net::SocketAddr, local_id: u64, remote_id: u64)->Connection {
        Connection::with_remote_id_and_state(address, local_id, remote_id, ConnectionState::Initialized)
    }

    fn with_remote_id_and_state(address: net::SocketAddr, local_id: u64, remote_id: u64, state: ConnectionState)->Connection {
        Connection {
            state: state,
            local_id: local_id,
            remote_id: remote_id,
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

    pub fn heartbeat<T: PacketSender>(&mut self, sender: &mut T) {
    }
}
