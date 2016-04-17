use super::*;
use super::super::packets::{self, Encodable, Decodable};
use super::super::responders;
use crc::crc32;
use byteorder::{self, BigEndian, ByteOrder};
use std::collections;
use std::net;
use mio;
use mio::udp;

const SOCKET_TOKEN: mio::Token = mio::Token(0);

pub struct MioHandlerState<'a> {
    socket: &'a udp::UdpSocket,
    incoming_packet_buffer: [u8; 1000],
    outgoing_packet_buffer: [u8; 1000],
}

pub struct MioHandler<'a> {
    state: MioHandlerState<'a>,
    connections: collections::HashMap<net::SocketAddr, Connection>,
    next_connection_id: u64,
    status_responder: responders::StatusResponder,
}

impl<'a> MioHandler<'a> {
    pub fn new(socket: &'a udp::UdpSocket)->MioHandler<'a> {
        MioHandler {
            state: MioHandlerState {
                socket: socket,
                incoming_packet_buffer: [0u8; 1000],
                outgoing_packet_buffer: [0u8; 1000],
            },
            connections: collections::HashMap::new(),
            next_connection_id: 0,
            status_responder: responders::StatusResponder::new(true, packets::PROTOCOL_VERSION, &[""; 0]),
        }
    }

    fn got_packet(&mut self, size: usize, address: net::SocketAddr) {
        if size == 0 {return;}
        let maybe_packet = {
            let slice = &self.state.incoming_packet_buffer[0..size];
            let computed_checksum = crc32::checksum_castagnoli(&slice[4..]);
            let expected_checksum = BigEndian::read_u32(&slice[..4]);
            if computed_checksum != expected_checksum {Err(packets::PacketDecodingError::Invalid)}
            else {packets::decode_packet(&slice[4..])}
        };
        if let Err(_) = maybe_packet {return;}
        let packet = maybe_packet.unwrap();
        if let Some(ref mut conn) = self.connections.get_mut(&address) {
            if conn.handle_incoming_packet(&packet, &mut self.state) {return;}
        }
        if self.status_responder.handle_incoming_packet_connectionless(&packet, address, &mut self.state) {return;}
        //if we get here, it's a special case.
        match packet {
            packets::Packet::Connect => {
                if let Some(_) = self.connections.get(&address) {return;}
                let id = self.next_connection_id;
                self.next_connection_id += 1;
                let conn = Connection::new(id, address);
                self.connections.insert(address, conn);
                self.state.send(&packets::Packet::Connected(id), address);
            },
            _ => {}
        }
    }
}

impl<'a> PacketSender for MioHandlerState<'a> {
    fn send(&mut self, packet: &packets::Packet, address: net::SocketAddr)->bool {
        if let Ok(size) = packets::encode_packet(packet, &mut self.outgoing_packet_buffer[4..]) {
            let checksum = crc32::checksum_castagnoli(&self.outgoing_packet_buffer[4..size]);
            BigEndian::write_u32(&mut self.outgoing_packet_buffer[..4], checksum);
            if let Ok(Some(sent_bytes)) = self.socket.send_to(&self.outgoing_packet_buffer[..4+size], &address) {
                if sent_bytes == 4+size {return true;}
                else {return false;}
            }
            else {return false;}
        }
        else {return false;};
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
            let result = self.state.socket.recv_from(&mut self.state.incoming_packet_buffer);
            if let Ok(Some((size, address))) = result {
                self.got_packet(size, address);
            }
        }
    }
}
