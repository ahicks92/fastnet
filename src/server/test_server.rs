#![macro_use]

use server::*;
use packets::*;
use std::net;

#[derive(Default, Debug)]
pub struct TestServer {
    pub sent_packets: Vec<(net::IpAddr, u16, Packet)>,
    pub established_connections: Vec<(net::IpAddr, u16)>,
}

impl TestServer {
    pub fn new()->TestServer {
        return TestServer::default();
    }
}

impl Server for TestServer {
    fn send(&mut self, packet: &Packet, ip: net::IpAddr, port: u16) {
        self.sent_packets.push((ip, port, packet.clone()));
    }

    fn make_connection(&mut self, ip: net::IpAddr, port: u16)->Result<u32, String> {
        self.established_connections.push((ip, port));
        Ok(1)
    }
}
