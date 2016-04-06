use super::*;
use super::super::server::{self, Server};
use super::super::test_server;
use super::super::packets::*;
use std::collections;
use std::iter::{self, Iterator, IntoIterator};
use std::convert;
use std::net;

pub struct StatusResponder {
    listening: bool,
    version: String,
    supported_extensions: collections::HashSet<String>,
}

impl StatusResponder {
    pub fn new<T>(listening: bool, version: &str, supported_extensions: &[T])->StatusResponder
    where T: convert::Into<String>+Clone,
    {
        let mut set = collections::HashSet::<String>::new();
        for i in supported_extensions.into_iter() {
            set.insert(i.clone().into());
        }
        StatusResponder {
            listening: listening,
            version: version.to_string(),
            supported_extensions: set,
        }
    }
}

impl PacketResponder for StatusResponder {
    fn handle_incoming_packet_always<T: server::Server>(&mut self, packet: &Packet, ip: net::IpAddr, server: &mut T)->bool {
        match *packet {
            Packet::StatusRequest(ref req) => {
                server.send(
                &match *req {
                    StatusRequest::FastnetQuery => Packet::StatusResponse(StatusResponse::FastnetResponse(self.listening)),
                    StatusRequest::VersionQuery => Packet::StatusResponse(StatusResponse::VersionResponse(self.version.clone())),
                    StatusRequest::ExtensionQuery(ref name) => {
                        let supported = self.supported_extensions.contains(name);
                        Packet::StatusResponse(StatusResponse::ExtensionResponse{name: name.clone(), supported: supported})
                    }
                }, &ip);
                true
            },
            _ => false
        }
    }
}

responder_test!(test_status_responder, |server: &mut test_server::TestServer, ip: net::IpAddr| {
    let mut responder = StatusResponder::new(true, "1.0", &["test_atest"]);
    responder.handle_incoming_packet_always(&Packet::StatusRequest(StatusRequest::FastnetQuery), ip, server);
    responder.handle_incoming_packet_always(&Packet::StatusRequest(StatusRequest::VersionQuery), ip, server);
    responder.handle_incoming_packet_always(&Packet::StatusRequest(StatusRequest::ExtensionQuery("test_atest".to_string())), ip, server);
},
Packet::StatusResponse(StatusResponse::FastnetResponse(true)),
Packet::StatusResponse(StatusResponse::VersionResponse("1.0".to_string())),
Packet::StatusResponse(StatusResponse::ExtensionResponse{name: "test_atest".to_string(), supported: true})
);
