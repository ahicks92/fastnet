use super::*;
use super::super::{Handler};
use super::super::packets::{self, Encodable, Decodable};
use super::super::status_translator;
use crc::crc32;
use byteorder::{self, BigEndian, ByteOrder};
use std::collections;
use std::net;
use std::thread;
use std::io;
use std::sync::mpsc;
use mio;
use mio::udp;

const SOCKET_TOKEN: mio::Token = mio::Token(0);

#[derive(Debug, Copy, Clone)]
pub enum TimeoutTypes {
    Timeout1000,
    Timeout200,
}

pub enum MioHandlerCommand<H: Handler> {
    DoCall(Box<fn(&mut MioHandler<H>)>),
}

/*This doesn't have a good name.

Basically it exists so that we can pass some stuff around without making the borrow checker mad.  Primarily it "provides" services, so we call it for that.*/
pub struct MioServiceProvider<'a, H: Handler> {
    pub socket: &'a udp::UdpSocket,
    pub incoming_packet_buffer: [u8; 1000],
    pub outgoing_packet_buffer: [u8; 1000],
    pub handler: H,
}

pub struct MioHandler<'a, H: Handler> {
    service: MioServiceProvider<'a, H>,
    connections: collections::HashMap<net::SocketAddr, Connection>,
    next_connection_id: u64,
}

impl<'a, H: Handler> MioHandler<'a, H> {
    pub fn new(socket: &'a udp::UdpSocket, handler: H)->MioHandler<'a, H> {
        MioHandler {
            service: MioServiceProvider {
                socket: socket,
                incoming_packet_buffer: [0u8; 1000],
                outgoing_packet_buffer: [0u8; 1000],
                handler: handler,
            },
            connections: collections::HashMap::new(),
            next_connection_id: 1,
        }
    }

    fn got_packet(&mut self, size: usize, address: net::SocketAddr) {
        if size == 0 {return;}
        let maybe_packet = {
            let slice = &self.service.incoming_packet_buffer[0..size];
            let computed_checksum = crc32::checksum_castagnoli(&slice[4..]);
            let expected_checksum = BigEndian::read_u32(&slice[..4]);
            if computed_checksum != expected_checksum {Err(packets::PacketDecodingError::Invalid)}
            else {packets::decode_packet(&slice[4..])}
        };
        if let Err(_) = maybe_packet {return;}
        let packet = maybe_packet.unwrap();
        if let Some(ref mut conn) = self.connections.get_mut(&address) {
            if conn.handle_incoming_packet(&packet, &mut self.service) {return;}
        }
        match packet {
            packets ::Packet::Connect => {
                if let Some(_) = self.connections.get(&address) {return;}
                let id = self.next_connection_id;
                self.next_connection_id += 1;
                let conn = Connection::new(address, id);
                self.connections.insert(address, conn);
                self.service.send(&packets::Packet::Connected(id), address);
            },
            packets::Packet::StatusRequest(ref req) => {
                self.service.send(&packets::Packet::StatusResponse(status_translator::translate(req)), address);
            },
            _ => {}
        }
    }
}

impl<'A, H: Handler> MioServiceProvider<'A, H> {
    pub fn send(&mut self, packet: &packets::Packet, address: net::SocketAddr)->bool {
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

impl<'a, H: Handler+Send> mio::Handler for MioHandler<'a, H> {
    type Timeout = TimeoutTypes;
    type Message = MioHandlerCommand<H>;

    fn ready(&mut self, event_loop: &mut mio::EventLoop<Self>, token: mio::Token, events: mio::EventSet) {
        //We only have one socket, so can avoid the match on the token.
        if events.is_error() {
            //We need to do something sensible here, probably a callback with whatever state we can get.
        }
        if events.is_readable() {
            let result = self.service.socket.recv_from(&mut self.service.incoming_packet_buffer);
            if let Ok(Some((size, address))) = result {
                self.got_packet(size, address);
            }
        }
    }

    fn timeout(&mut self, event_loop: &mut mio::EventLoop<Self>, timeout: Self::Timeout) {
        //Rust isn't smart enough to realize that closures only borrow a field, so we pull it out here to satisfy the borrow checker.
        let sender = &mut self.service;
        let rereg = match timeout {
            TimeoutTypes::Timeout200 => {
                self.connections.iter_mut().map(|x| x.1.tick200(sender));
                200
            },
            TimeoutTypes::Timeout1000 => {
                self.connections.iter_mut().map(|x| x.1.tick1000(sender));
                1000
            },
        };
        event_loop.timeout_ms(timeout, rereg).unwrap();
    }
}

fn mio_server_thread< H: Handler+Send>(address: net::SocketAddr, handler: H, notify_created: mpsc::Sender<Result<mio::Sender<MioHandlerCommand<H>>, io::Error>>) {
    let maybe_socket = match address {
        net::SocketAddr::V4(_) => udp::UdpSocket::v4(),
        net::SocketAddr::V6(_) => udp::UdpSocket::v6()
    };
    if let Err(what) = maybe_socket {
        notify_created.send(Err(what)).unwrap();
        return;
    }
    let socket = maybe_socket.unwrap();
    if let  Err(what) = socket.bind(&address) {
        notify_created.send(Err(what)).unwrap();
        return;
    }
    let maybe_loop  = mio::EventLoop::new();
    if let Err(what) = maybe_loop {
        notify_created.send(Err(what)).unwrap();
        return;
    }
    let mut event_loop = maybe_loop.unwrap();
    let mut handler = MioHandler::new(&socket, handler);
    if let Err(what)  = event_loop.register(&socket, SOCKET_TOKEN, mio::EventSet::all(), mio::PollOpt::all()) {
        notify_created.send(Err(what)).unwrap();
        return;
    }
    let timer_error = Err(io::Error::new(io::ErrorKind::Other, "Couldn't create the timer."));
    if let Err(_) = event_loop.timeout_ms(TimeoutTypes::Timeout1000, 1000) {
        notify_created.send(timer_error).unwrap();
        return;
    }
    if let Err(_) = event_loop.timeout_ms(TimeoutTypes::Timeout200, 200) {
        notify_created.send(timer_error);
        return;
    }
    let sender = event_loop.channel();
    notify_created.send(Ok(sender));
    event_loop.run(&mut handler);
}

pub struct MioServer<H: Handler> {
    thread: thread::JoinHandle<()>,
    sender: mio::Sender<MioHandlerCommand<H>>,
}

impl<H: Handler+Send+'static> MioServer<H> {
    fn new(address: net::SocketAddr, handler: H)->Result<MioServer<H>, io::Error> {
        let (sender, receiver) = mpsc::channel();
        let join_handle = thread::spawn(move || mio_server_thread(address, handler, sender));
        let message_sender = try!(receiver.recv().unwrap());
        Ok(MioServer {
            thread: join_handle,
            sender: message_sender,
        })
    }
}
