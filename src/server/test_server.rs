#![macro_use]

use server::*;
use packets::*;
use std::net;

#[derive(Default, Debug)]
pub struct TestServer {
    pub sent_packets: Vec<(net::SocketAddr, Packet)>,
    pub established_connections: Vec<net::SocketAddr>,
}

impl TestServer {
    pub fn new()->TestServer {
        return TestServer::default();
    }
}

impl Server for TestServer {
    fn send(&mut self, packet: &Packet, address: net::SocketAddr) {
        self.sent_packets.push((address, packet.clone()));
    }

    fn make_connection(&mut self, address: net::SocketAddr)->Result<u32, String> {
        self.established_connections.push(address);
        Ok(1)
    }
}
