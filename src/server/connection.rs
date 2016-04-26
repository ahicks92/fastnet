use super::*;
use super::super::packets::*;
use super::super::async;
use super::super::status_translator;
use std::net;


#[derive(Debug, Copy, Clone)]
pub enum ConnectionState {
    Establishing{listening: bool, compatible_version: bool, attempts: u32, request_id: Option<u64>},
    Established,
    Closing{request_id: Option<u64>},
    Closed,
}

#[derive(Debug)]
pub struct Connection {
    pub state: ConnectionState,
    pub local_id: u64,
    pub remote_id: u64,
    pub address: net::SocketAddr,
    pub received_packets: u64,
    pub sent_packets: u64,
    pub heartbeat_counter: u64,
}

const MAX_STATUS_ATTEMPTS: u32 = 10;
const MAX_CONNECTION_ATTEMPTS:u32 = 25; //5000 ms divided by 200 ms per attempt, see spec.

impl Connection {

    pub fn new(address: net::SocketAddr, local_id: u64)->Connection {
        Connection {
            state: ConnectionState::Closed,
            local_id: local_id,
            remote_id: 0,
            address: address,
            sent_packets: 0,
            received_packets: 0,
            heartbeat_counter: 0,
        }
    }

    pub fn send<H: async::Handler>(&mut self, packet: &Packet, service: &mut MioServiceProvider<H>)->bool {
        self.sent_packets += 1;
        service.send(packet, self.address)
    }

    pub fn handle_incoming_packet<H: async::Handler>(&mut self, packet: &Packet, service: &mut MioServiceProvider<H>)->bool {
        self.received_packets += 1; //Always.
        match *packet {
            Packet::Echo(id) => {
                self.send(packet, service);
                true
            },
            Packet::Heartbeat{counter: c, sent: s, received: r} => {
                true
            },
            Packet::Connected(id) => {
                self.handle_connected(id, service);
                true
            },
            Packet::Aborted(ref message) => {
                self.handle_aborted(message, service);
                true
            },
            _ => false
        }
    }

    fn handle_connected<H: async::Handler>(&mut self, id: u64, service: &mut MioServiceProvider<H>) {
        if let ConnectionState::Establishing{listening, compatible_version, request_id, ..} = self.state {
            if listening && compatible_version {
                self.remote_id = id;
                self.state = ConnectionState::Established;
            }
            service.handler.connected(self.local_id, request_id);
        }
        //Otherwise, we shouldn't be receiving this yet so just drop it.
    }

    fn handle_aborted<H: async::Handler>(&mut self, message: &str, service: &mut MioServiceProvider<H>) {
        self.state = ConnectionState::Closed;
        //TODO: notify the user.
    }

    fn handle_status_response<H: async::Handler>(&mut self, resp: &StatusResponse, service: &mut MioServiceProvider<H>) {
        if let ConnectionState::Establishing{mut listening, mut compatible_version, mut attempts, request_id} = self.state {
            match *resp {
                StatusResponse::FastnetResponse(new_listening) if listening == false => {
                    if new_listening == false {
                        if let Some(id) = request_id {service.handler.request_failed(id, async::Error::NotListening);}
                        self.state = ConnectionState::Closed;
                        return;
                    }
                    listening = true;
                },
                StatusResponse::VersionResponse(ref v) if compatible_version == false => {
                    if v.eq(status_translator::PROTOCOL_VERSION) == false {
                        if let Some(id) = request_id {service.handler.request_failed(id, async::Error::IncompatibleVersions)}
                        self.state = ConnectionState::Closed;
                        return;
                    }
                    compatible_version = true;
                }
                _ => {}
            }
            self.state = ConnectionState::Establishing{attempts: 0, listening: listening, compatible_version: compatible_version, request_id: request_id};
        }
    }

    pub fn tick1000<H: async::Handler>(&mut self, service: &mut MioServiceProvider<H>) {
        if let ConnectionState::Established = self.state {
            let heartbeat = Packet::Heartbeat{counter: self.heartbeat_counter, sent: self.sent_packets, received: self.received_packets};
            self.heartbeat_counter += 1;
            self.send(&heartbeat, service);
        }
    }

    pub fn tick200<H: async::Handler>(&mut self, service: &mut MioServiceProvider<H>) {
        match self.state {
            ConnectionState::Establishing{mut attempts, listening, compatible_version, request_id} => {
                attempts += 1;
                if listening == false {
                    if attempts > MAX_STATUS_ATTEMPTS {
                        if let Some(id) = request_id {service.handler.request_failed(id, async::Error::TimedOut);}
                        self.state = ConnectionState::Closed;
                        return;
                    }
                    service.send(&Packet::StatusRequest(StatusRequest::FastnetQuery), self.address);
                }
                else if compatible_version == false {
                    if attempts > MAX_STATUS_ATTEMPTS {
                        if let Some(id) = request_id {service.handler.request_failed(id, async::Error::TimedOut);}
                        self.state = ConnectionState::Closed;
                        return;
                    }
                    service.send(&Packet::StatusRequest(StatusRequest::VersionQuery), self.address);
                }
                else {
                    if attempts > MAX_CONNECTION_ATTEMPTS {
                        if let Some(id) = request_id {service.handler.request_failed(id, async::Error::TimedOut);}
                        self.state = ConnectionState::Closed;
                        return;
                    }
                    service.send(&Packet::Connect, self.address);
                }
            },
            _ => {},
        }
    }
}
