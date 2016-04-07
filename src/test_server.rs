#![macro_use]

use server::*;
use packets::*;
use std::net;

#[derive(Default, Debug)]
pub struct TestServer {
    pub sent_packets: Vec<(net::IpAddr, Packet)>,
    pub established_connections: Vec<net::IpAddr>,
}

impl TestServer {
    pub fn new()->TestServer {
        return TestServer::default();
    }
}

impl Server for TestServer {
    fn send(&mut self, packet: Packet, ip: net::IpAddr) {
        self.sent_packets.push((ip, packet));
    }
    fn make_connection(&mut self, ip: net::IpAddr)->Result<u32, String> {
        self.established_connections.push(ip);
        Ok(1)
    }
}
