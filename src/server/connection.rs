use super::*;
use super::super::packets::*;
use super::super::status_translator;
use std::net;


#[derive(Debug, Copy, Clone)]
pub enum ConnectionState {
    Establishing{listening: bool, compatible_version: bool, attempts: u32},
    Established,
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

    pub fn send(&mut self, packet: &Packet, service: &mut MioServiceProvider)->bool {
        self.sent_packets += 1;
        service.send(packet, self.address)
    }

    pub fn handle_incoming_packet(&mut self, packet: &Packet, service: &mut MioServiceProvider)->bool {
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
                self.handle_connected(id);
                true
            },
            Packet::Aborted(ref message) => {
                self.handle_aborted(message);
                true
            },
            _ => false
        }
    }

    fn handle_connected(&mut self, id: u64) {
        if let ConnectionState::Establishing{listening, compatible_version, ..} = self.state {
            if listening && compatible_version {
                self.remote_id = id;
                self.state = ConnectionState::Established;
            }
        }
        //Otherwise, we shouldn't be receiving this yet so just drop it.
    }

    fn handle_aborted(&mut self, message: &str) {
        self.state = ConnectionState::Closed;
        //TODO: notify the user.
    }

    fn handle_status_response(&mut self, resp: &StatusResponse) {
        if let ConnectionState::Establishing{mut listening, mut compatible_version, mut attempts} = self.state {
            match *resp {
                StatusResponse::FastnetResponse(new_listening) if listening == false => {
                    if new_listening == false {
                        self.state = ConnectionState::Closed;
                        return;
                    }
                    listening = true;
                },
                StatusResponse::VersionResponse(ref v) if compatible_version == false => {
                    if v.eq(status_translator::PROTOCOL_VERSION) == false {
                        self.state = ConnectionState::Closed;
                        return;
                    }
                    compatible_version = true;
                }
                _ => {}
            }
            self.state = ConnectionState::Establishing{attempts: 0, listening: listening, compatible_version: compatible_version};
        }
    }

    pub fn tick1000(&mut self, service: &mut MioServiceProvider) {
        let heartbeat = Packet::Heartbeat{counter: self.heartbeat_counter, sent: self.sent_packets, received: self.received_packets};
        self.heartbeat_counter += 1;
        self.send(&heartbeat, service);
    }

    pub fn tick200(&mut self, service: &mut MioServiceProvider) {
        match self.state {
            ConnectionState::Establishing{mut attempts, listening, compatible_version} => {
                attempts += 1;
                if listening == false {
                    if attempts > MAX_STATUS_ATTEMPTS {
                        self.state = ConnectionState::Closed;
                        return;
                    }
                    service.send(&Packet::StatusRequest(StatusRequest::FastnetQuery), self.address);
                }
                else if compatible_version == false {
                    if attempts > MAX_STATUS_ATTEMPTS {
                        self.state = ConnectionState::Closed;
                        return;
                    }
                    service.send(&Packet::StatusRequest(StatusRequest::VersionQuery), self.address);
                }
                else {
                    if attempts > MAX_CONNECTION_ATTEMPTS {
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
