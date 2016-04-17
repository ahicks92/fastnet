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
    next_connection_id: u32,
}

pub struct MioHandler<'a> {
    state: MioHandlerState<'a>,
    connections: collections::HashMap<net::SocketAddr, Connection>,
    status_responder: responders::StatusResponder,
    connection_responder: responders::ConnectionResponder,
}

impl<'a> MioHandler<'a> {
    pub fn new(socket: &'a udp::UdpSocket)->MioHandler<'a> {
        MioHandler {
            state: MioHandlerState {
                socket: socket,
                incoming_packet_buffer: [0u8; 1000],
                outgoing_packet_buffer: [0u8; 1000],
                next_connection_id: 0,
            },
            connections: collections::HashMap::new(),
            status_responder: responders::StatusResponder::new(true, packets::PROTOCOL_VERSION, &[""; 0]),
            connection_responder: responders::ConnectionResponder::new(),
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
        self.status_responder.handle_incoming_packet_connectionless(&packet, address, &mut self.state)
        || self.connection_responder.handle_incoming_packet_connectionless(&packet, address, &mut self.state);
    }
}

impl<'a> Server for MioHandlerState<'a> {
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
            let result = self.state.socket.recv_from(&mut self.state.incoming_packet_buffer);
            if let Ok(Some((size, address))) = result {
                self.got_packet(size, address);
            }
        }
    }
}
