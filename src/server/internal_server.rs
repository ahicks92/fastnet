/*! An internal server.

This server does bookkeeping and the like, but doesn't handle I/O.  TestServer and MioServer manipulate it as needed.*/

use super::*;
use super::super::packets;
use crc;
use std::collections;
use std::net;

pub struct InternalServer {
    outgoing_packets: collections::VecDeque<packets::Packet>,
    connections: collections::HashMap<(net::IpAddr, u16), Connection>,
    next_connection_id: u32, //TODO: a counter is not sufficient for long-running programs.
}

impl InternalServer {
    pub fn new()->InternalServer {
        InternalServer {
            connections: collections::HashMap::new(),
            outgoing_packets: collections::VecDeque::with_capacity(100),
            next_connection_id: 1, //0 is the server.
        }
    }

    pub fn incoming_packet(&mut self, packet: &packets::Packet, ip: net::IpAddr) {
        //no-op for now.
    }

    fn get_outgoing_packet_count(&self)->usize {
        self.outgoing_packets.len()
    }

    fn pop_outgoing_packet(&mut self)->Option<packets::Packet> {
        self.outgoing_packets.pop_front()
    }
}

impl Server for InternalServer {
    fn send(&mut self, packet: &packets::Packet, ip: net::IpAddr, port: u16) {
    }

    fn make_connection(&mut self, ip: net::IpAddr, port: u16)->Result<u32, String> {
        let id = self.next_connection_id;
        self.next_connection_id += 1;
        let conn = Connection::new(id, ip, port);
        self.connections.insert((ip, port), conn);
        Ok(id)
    }
}
