pub use self::encoder::*;

mod encoder;
mod encoder_tests;

pub enum Packet {
    //Status request and response (channel -1)
    StatusRequest(StatusRequest),
    StatusResponse(StatusResponse),

    //Connection handshake (also channel -1).
    Connect,
    Connected(u32),
    Aborted(String),
    
    //Heartbeat (channel -2).
    Heartbeat{counter: u32, sent: u64, received: u64},

    Echo(i16),
}

pub enum StatusRequest {
    FastnetQuery,
    VersionQuery,
    ExtensionQuery(String),
}

pub enum StatusResponse {
    FastnetResponse(u8),
    VersionResponse(String),
    ExtensionResponse {name: String, supported: bool},
}


pub const CONNECTION_CHANNEL: i16 = -1;
pub const HEARTBEAT_CHANNEL: i16 = -2;
pub const ECHO_CHANNEL: i16 = -3;
