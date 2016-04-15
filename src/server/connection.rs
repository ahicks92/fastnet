use super::*;
use super::super::{packets, responders};
use std::net;

#[derive(Debug)]
pub struct ConnectionState {
    pub id: u32,
    pub address: net::SocketAddr,
}

#[derive(Debug)]
pub struct Connection {
    state: ConnectionState,
    pub heartbeat_responder: responders::HeartbeatResponder,
    pub echo_responder: responders::EchoResponder,
}

impl Connection {
    pub fn new(id: u32, address: net::SocketAddr)->Connection {
        Connection {
            state: ConnectionState {
                id: id,
                address: address,
            },
            heartbeat_responder: responders::HeartbeatResponder::new(),
            echo_responder: responders::EchoResponder::new(),
        }
    }
}
