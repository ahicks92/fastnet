use packets;
use std::net;

mod mio_server;
mod connection;

pub use self::mio_server::*;
pub use self::connection::*;

pub trait PacketSender {
    //Send a packet. Returns false if we didn't actually send it.
    fn send(&mut self, packet: &packets::Packet, address: net::SocketAddr)->bool;
}