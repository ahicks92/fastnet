use super::*;
use super::super::{packets, responders};
use std::net;

#[derive(Debug)]
pub struct ConnectionState {
    pub id: u64,
    pub address: net::SocketAddr,
}

#[derive(Debug)]
pub struct Connection {
    pub state: ConnectionState,
    pub heartbeat_responder: responders::HeartbeatResponder,
    pub echo_responder: responders::EchoResponder,
}

impl Connection {
    pub fn new(id: u64, address: net::SocketAddr)->Connection {
        Connection {
            state: ConnectionState {
                id: id,
                address: address,
            },
            heartbeat_responder: responders::HeartbeatResponder::new(),
            echo_responder: responders::EchoResponder::new(),
        }
    }

    pub fn handle_incoming_packet<T: PacketSender>(&mut self, packet: &packets::Packet, server: &mut T)->bool {
        self.heartbeat_responder.handle_incoming_packet(packet, &mut self.state, server)
        || self.echo_responder.handle_incoming_packet(packet, &mut self.state, server)
    }
}
