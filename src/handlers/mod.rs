use super::packets;
use super::server;
use mio;

mod connection;
mod echo;
mod heartbeat;
mod status;

pub use self::connection::*;
pub use self::echo::*;
pub use self::heartbeat::*;
pub use self::status::*;

pub trait PacketHandler {
    //Return true if and only if this handler handles the packet.
    //This function is called if and only if the packet is destined for an already-established connection and this handler is associated with it.
    //The server handles the initial association when connections are created.
    fn handle_incoming_packet(&mut self, packet: &packets::Packet, server: &mut server::Server)->bool {
        false
    }
    //This variant is called when the packet is not for a connection.
    fn handle_incoming_packet_connectionless(&mut self, packet: &packets::Packet, ip: &mio::IpAddr, server: &mut server::Server)->bool {
        false
    }
    //This variant is called in both cases, and is used primarily for the status responses.
    //It is called before either handle_incoming_packet or handle_incoming_packet_connectionless.
    fn handle_incoming_packet_always(&mut self, packet: &packets::Packet, ip: &mio::IpAddr, server: &mut server::Server)->bool {
        false
    }
    //This happens if and only if get_tick_frequency returns a time.
    fn tick(&mut self, server: &mut server::Server) {
    }
    //Return a time in MS.
    fn get_tick_frequency()->Option<u32> {
        None
    }
}
