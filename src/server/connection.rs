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
    pub received_packets: u64,
    pub sent_packets: u64,
    pub heartbeat_counter: u64,
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
            sent_packets: 0,
            received_packets: 0,
            heartbeat_counter: 0,
        }
    }

    pub fn send<T: PacketSender>(&mut self, packet: &packets::Packet, sender: &mut T)->bool {
        self.sent_packets += 1;
        sender.send(packet, self.address)
    }

    pub fn handle_incoming_packet<T: PacketSender>(&mut self, packet: &packets::Packet, sender: &mut T)->bool {
        self.received_packets += 1; //Always.
        match *packet {
            Packet::Echo(id) => {
                self.send(packet, sender);
                true
            },
            Packet::Heartbeat{counter: c, sent: s, received: r} => {
                true
            },
            _ => false
        }
    }

    pub fn heartbeat<T: PacketSender>(&mut self, sender: &mut T) {
        let heartbeat = packets::Packet::Heartbeat{counter: self.heartbeat_counter, sent: self.sent_packets, received: self.received_packets};
        self.send(&heartbeat, sender);
    }
}
