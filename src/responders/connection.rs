use super::*;
use super::super::server::*;
use super::super::packets;
use std::net;

#[derive(Debug, Default)]
pub struct ConnectionResponder;

impl ConnectionResponder {
    pub fn new()->ConnectionResponder {
        ConnectionResponder::default()
    }
}

impl ConnectionlessPacketResponder for ConnectionResponder {
    fn handle_incoming_packet_connectionless<T: Server>(&mut self, packet: &packets::Packet, address: net::SocketAddr, server: &mut T)->bool {
        match *packet {
            packets::Packet::Connect => {
                let maybe_connected = server.make_connection(address);
                let response = match maybe_connected {
                    Ok(id) => packets::Packet::Connected(id),
                    Err(ref msg) => packets::Packet::Aborted(msg.clone())
                };
                server.send(&response, address);
                true
            },
            _ => false
        }
    }
}

responder_test!(test_connection_responder, |server: &mut TestServer, connection: &mut ConnectionState, address: net::SocketAddr| {
    let mut handler = ConnectionResponder::new();
    handler.handle_incoming_packet_connectionless(&packets::Packet::Connect, address, server);
    assert_eq!(server.established_connections.len(), 1);
    assert_eq!(server.established_connections[0], address);
},
packets::Packet::Connected(1) //test server always does 1 at the connection id.
);