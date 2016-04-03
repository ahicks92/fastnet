#![macro_use]

use server::*;
use packets::*;
use mio;

pub struct TestServer {
    sent_packets: Vec<(mio::IpAddr, Packet)>,
    established_connections: Vec<mio::IpAddr>,
}

impl Server for TestServer {
    fn send(&mut self, packet: &Packet, ip: &mio::IpAddr) {
        //Apparently mio doesn't make IpAddr copyable or cloneable.
        let pushable_ip = match *ip {
            mio::IpAddr::V4(x) => mio::IpAddr::V4(x),
            mio::IpAddr::V6(x) => mio::IpAddr::V6(x),
        };
        self.sent_packets.push((pushable_ip, packet.clone()));
    }
    fn make_connection(&mut self, ip: &mio::IpAddr) {
        let pushable_ip = match *ip {
            mio::IpAddr::V4(x) => mio::IpAddr::V4(x),
            mio::IpAddr::V6(x) => mio::IpAddr::V6(x),
        };
        self.established_connections.push(pushable_ip);
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