use super::*;
use super::super::{packets, responders};
use std::net;

#[derive(Debug)]
pub struct Connection {
    pub id: u32,
    pub address: net::SocketAddr,
    pub heartbeat_responder: responders::HeartbeatResponder,
    pub echo_responder: responders::EchoResponder,
}

impl Connection {
    pub fn new(id: u32, address: net::SocketAddr)->Connection {
        Connection {
            id: id,
            address: address,
            heartbeat_responder: responders::HeartbeatResponder::new(),
            echo_responder: responders::EchoResponder::new(),
                    }
    }
}
