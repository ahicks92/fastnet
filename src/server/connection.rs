use super::*;
use super::super::packets::{self, Packet};
use std::net;


#[derive(Debug, Copy, Clone)]
pub enum ConnectionState {
    Establishing{listening: bool, compatible_version: bool, attempts: u32},
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

const MAX_STATUS_ATTEMPTS: u32 = 10;
const MAX_CONNECTION_ATTEMPTS:u32 = 25; //5000 ms divided by 200 ms per attempt, see spec.

impl Connection {

    pub fn new(address: net::SocketAddr, local_id: u64)->Connection {
        Connection {
            state: ConnectionState::Closed,
            local_id: local_id,
            remote_id: 0,
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

    pub fn tick1000<T: PacketSender>(&mut self, sender: &mut T) {
        let heartbeat = packets::Packet::Heartbeat{counter: self.heartbeat_counter, sent: self.sent_packets, received: self.received_packets};
        self.send(&heartbeat, sender);
    }

    pub fn tick200<T: PacketSender>(&mut self, sender: &mut T) {
        match self.state {
            ConnectionState::Establishing{mut attempts, listening, compatible_version} => {
                attempts += 1;
                if listening == false {
                    if attempts > MAX_STATUS_ATTEMPTS {
                        self.state = ConnectionState::Closed;
                        return;
                    }
                    sender.send(&packets::Packet::StatusRequest(packets::StatusRequest::FastnetQuery), self.address);
                }
                else if compatible_version == false {
                    if attempts > MAX_STATUS_ATTEMPTS {
                        self.state = ConnectionState::Closed;
                        return;
                    }
                    sender.send(&packets::Packet::StatusRequest(packets::StatusRequest::VersionQuery), self.address);
                }
                else {
                    if attempts > MAX_CONNECTION_ATTEMPTS {
                        self.state = ConnectionState::Closed;
                        return;
                    }
                    sender.send(&packets::Packet::Connect, self.address);
                }
            },
            _ => {},
        }
    }
}
