use super::*;
use super::super::packets;
use crc;
use std::collections;
use std::net;
use mio;
use mio::udp;

const SOCKET_TOKEN: mio::Token = mio::Token(0);

//The actual server consists of this, the event loop, and the socket.
//The struct is later in this file.
pub struct MioHandler<'a> {
    outgoing_packets: collections::VecDeque<packets::Packet>,
    connections: collections::HashMap<net::SocketAddr, Connection>,
    next_connection_id: u32, //TODO: a counter is not sufficient for long-running programs.

    socket: &'a udp::UdpSocket,
    incoming_packet_buffer: [u8; 1000],
    outgoing_packet_buffer: [u8; 1000],
}

impl<'a> MioHandler<'a> {
    pub fn new(socket: &'a udp::UdpSocket)->MioHandler<'a> {
        MioHandler {
            connections: collections::HashMap::new(),
            outgoing_packets: collections::VecDeque::with_capacity(100),
            next_connection_id: 0,
            socket: socket,
            incoming_packet_buffer: [0u8; 1000],
            outgoing_packet_buffer: [0u8; 1000],
        }
    }

    fn got_packet(&mut self, size: usize, address: net::SocketAddr) {
        if size == 0 {return};
        let slice = &self.incoming_packet_buffer[0..size];
        //Todo: finish.
    }
}

impl<'a> Server for MioHandler<'a> {
    fn send(&mut self, packet: &packets::Packet, address: net::SocketAddr) {
    }

    fn make_connection(&mut self, address: net::SocketAddr)->Result<u32, String> {
        let id = self.next_connection_id;
        self.next_connection_id += 1;
        let conn = Connection::new(id, address);
        self.connections.insert(address, conn);
        Ok(id)
    }
}

impl<'a> mio::Handler for MioHandler<'a> {
    type Timeout = ();
    type Message = ();

    fn ready(&mut self, event_loop: &mut mio::EventLoop<Self>, token: mio::Token, events: mio::EventSet) {
        //We only have one socket, so can avoid the match on the token.
        if events.is_error() {
            //We need to do something sensible here, probably a callback with whatever state we can get.
        }
        if events.is_readable() {
            let result = self.socket.recv_from(&mut self.incoming_packet_buffer);
            if let Ok(Some((size, address))) = result {
                self.got_packet(size, address);
            }
        }
    }
}
