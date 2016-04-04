#![macro_use]

use server::*;
use packets::*;
use std::net;

pub struct TestServer {
    sent_packets: Vec<(net::IpAddr, Packet)>,
    established_connections: Vec<net::IpAddr>,
}

impl Server for TestServer {
    fn send(&mut self, packet: &Packet, ip: &net::IpAddr) {
        self.sent_packets.push((*ip, packet.clone()));
    }
    fn make_connection(&mut self, ip: &net::IpAddr) {
        self.established_connections.push(*ip);
    }
}

macro_rules! assert_sent_packets {
    ($server: expr, $ip: expr, ($packets: expr),* ) => {
        {
            let i = $server.sent_packets.iter();
            $(assert_eq!(($ip, $packets), i.next().unwrap());)*
        }
    }
}
