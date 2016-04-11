use super::super::server::*;
use super::super::packets;
use super::*;
use super::super::packets::*;
use super::super::test_server;
use std::net;

#[derive(Default, Debug)]
pub struct HeartbeatResponder;

impl HeartbeatResponder {
    pub fn new()->HeartbeatResponder {
        HeartbeatResponder::default()
    }
}

impl ConnectedPacketResponder for HeartbeatResponder {
    fn handle_incoming_packet<T: Server>(&mut self, packet: &packets::Packet, server: &mut T)->bool {
        //For now, do nothing but swallow the heartbeat.
        if let packets::Packet::Heartbeat{counter: c, sent: s, received: r} = *packet {true}
        else {false}
    }
}

//We test that it's no-op anyway...
responder_test!(test_heartbeat_responder, |server, ip| {
    let mut responder = HeartbeatResponder::new();
    responder.handle_incoming_packet(&packets::Packet::Heartbeat{counter: 1, sent:2, received: 5}, server);
}, //nothing, but the macro needs the comma.
);
