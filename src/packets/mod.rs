/*! Provides packet encoding and decoding functionality, as well as the packet enum.

This module does not handle the checksum.  If it did, it would be incredibly difficult to write Fastnet tests.*/
pub use self::encoder::*;
pub use self::decoder::*;

mod encoder;
mod encoder_tests;
mod decoder;
mod decoder_tests;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Packet {
    //Status request and response (channel -1)
    StatusRequest(StatusRequest),
    StatusResponse(StatusResponse),

    //Connection handshake (also channel -1).
    Connect,
    Connected(u32),
    Aborted(String),
    
    //Heartbeat (channel -2).
    Heartbeat{counter: u64, sent: u64, received: u64},

    Echo(i16),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum StatusRequest {
    FastnetQuery,
    VersionQuery,
    ExtensionQuery(String),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum StatusResponse {
    FastnetResponse(bool),
    VersionResponse(String),
    ExtensionResponse {name: String, supported: bool},
}

pub const PROTOCOL_VERSION: &'static str = "1.0";
pub const CONNECTION_CHANNEL: i16 = -1;
pub const HEARTBEAT_CHANNEL: i16 = -2;
pub const ECHO_CHANNEL: i16 = -3;

pub const STATUS_REQUEST_SPECIFIER: u8 = 0;
pub const STATUS_RESPONSE_SPECIFIER: u8 = 1;
pub const CONNECT_SPECIFIER: u8 = 2;
pub const CONNECTED_SPECIFIER: u8 = 3;
pub const ABORTED_SPECIFIER: u8 = 4;

//These are used both for query and response.
pub const STATUS_FASTNET_SPECIFIER: u8 = 0;
pub const STATUS_VERSION_SPECIFIER: u8 = 1;
pub const STATUS_EXTENSION_SPECIFIER: u8 = 2;
