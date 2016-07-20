/*! Provides packet encoding and decoding functionality, as well as the packet enum.

This module does not handle the checksum.  If it did, it would be incredibly difficult to write Fastnet tests.*/
pub use self::encoder::*;
pub use self::decoder::*;
use uuid;
use std::cmp;

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
    Connect(uuid::Uuid),
    Connected(uuid::Uuid),
    Aborted(String),
    
    //Heartbeat (channel -2).
    Heartbeat{counter: u64, sent: u64, received: u64},

    Echo{endpoint: uuid::Uuid, uuid: uuid::Uuid},
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

//Flag bits for data packets, used in the impl of the struct.
pub const DATA_FRAME_START_BIT: u8 = 0;
pub const DATA_FRAME_END_BIT: u8 = 1;
pub const DATA_RELIABLE_BIT: u8 = 2;

/**Represents the part of a data packet that a channel must use to assemble packets.

The actual channel itself is stored in the enum variant.

These are ordered by sequence number, for use in trees.*/
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct DataPacket {
    sequence_number: u64,
    flags: u8,
    payload: Vec<u8>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DataPacketBuilder {
    sequence_number: u64,
    is_reliable: bool,
    is_start: bool,
    is_end: bool,
    payload: Vec<u8>,
}

impl DataPacketBuilder {
    /**Initial state is unreliable, mid-frame, and empty payload.*/
    pub fn new(sequence_number: u64)->DataPacketBuilder {
        DataPacketBuilder::with_payload(sequence_number, Vec::default())
    }

    /**Configures the builder for mid-frame, unreliable but using the specified payload.*/
    pub fn with_payload(sequence_number: u64, payload: Vec<u8>)->DataPacketBuilder {
        DataPacketBuilder {
            sequence_number: sequence_number,
            is_reliable: false,
            is_start: false,
            is_end: false,
            payload: payload,
        }
    }

    pub fn set_payload(&mut self, payload: Vec<u8>) {
        self.payload = payload;
    }

    pub fn set_reliable(&mut self, reliable: bool) {
        self.is_reliable = reliable;
    }

    pub fn set_start(&mut self, start: bool) {
        self.is_start = start;
    }

    pub fn set_end(&mut self, end: bool) {
        self.is_end = end;
    }

    pub fn set_sequence_number(&mut self, sequence_number: u64) {
        self.sequence_number = sequence_number;
    }

    pub fn build(self)->Option<DataPacket> {
        let start_flag  = (self.is_start as u8) << DATA_FRAME_START_BIT;
        let end_flag = (self.is_end as u8) << DATA_FRAME_END_BIT;
        let reliable_flag = (self.is_reliable as u8) << DATA_RELIABLE_BIT;
        let flags = start_flag | end_flag | reliable_flag;
        Some(DataPacket {
            sequence_number: self.sequence_number,
            flags: flags,
            payload: self.payload,
        })
    }
}

impl DataPacket {
    pub fn is_reliable(&self)->bool {
        (self.flags & DATA_RELIABLE_BIT) > 0
    }

    pub fn is_frame_start(&self)->bool {
        (self.flags & DATA_FRAME_START_BIT) > 0
    }

    pub fn is_frame_end(&self)->bool {
        (self.flags & DATA_FRAME_END_BIT) > 0
    }

    pub fn sequence_number(&self)->u64 {
        self.sequence_number
    }

    pub fn borrow_payload(&self)->&Vec<u8> {
        &self.payload
    }

    pub fn into_payload(self)->Vec<u8> {
        self.payload
    }
}