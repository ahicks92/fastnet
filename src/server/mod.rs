use packets;
use responders;
use std::net;

mod test_server;
mod mio_server;
mod connection;

pub use self::test_server::*;
pub use self::mio_server::*;
pub use self::connection::*;

pub trait Server {
    //Send a packet. Returns false if we didn't actually send it.
    fn send(&mut self, packet: &packets::Packet, address: net::SocketAddr)->bool;
    //Upgrade an ip address/port pair to a connection.
    fn make_connection(&mut self, address: net::SocketAddr)->Result<u32, String>;
}

/**Represents responders which are associated with connections.

The single method here should return true if the packet was handled, otherwise false.
The server tries all responders associated with a connection, then tries all responders not associated with any connection.
If a packet isn't handled by anything, it is dropped.*/
pub trait ConnectedPacketResponder {
    fn handle_incoming_packet<T: Server>(&mut self, packet: &packets::Packet, connection: &Connection, server: &mut T)->bool {
        false
    }
}

/**Represents responders which are not associated with connections.

The single method here should return true if the packet was handled, otherwise false.
The server tries all responders associated with a connection, then tries all responders not associated with any connection.
If a packet isn't handled by anything, it is dropped.*/
pub trait ConnectionlessPacketResponder {
    fn handle_incoming_packet_connectionless<T: Server>(&mut self, packet: &packets::Packet, address: net::SocketAddr, server: &mut T)->bool {
        false
    }
}
