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
    
    Data{chan: i16, packet: DataPacket},
    Ack{chan: i16, sequence_number: u64}
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

pub const DATA_PACKET_SPECIFIER: u8 = 0;
pub const ACK_PACKET_SPECIFIER: u8 = 1;

pub const FRAME_HEADER_SIZE: usize = 12; //64-bit sequence number and 32-bit length.

/**Represents the part of a data packet that a channel must use to assemble packets.

The actual channel itself is stored in the enum variant.

These are ordered by sequence number, for use in trees.*/
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct DataPacket {
    sequence_number: u64,
    flags: u8,
    payload: Vec<u8>,
    header: Option<FrameHeader>
}

//It would be nice to put this somewhere else, but we unfortunately can't.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default)]
pub struct FrameHeader {
    pub last_reliable_frame: u64,
    pub length: u32,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DataPacketBuilder {
    sequence_number: u64,
    is_reliable: bool,
    is_frame_start: bool,
    is_frame_end: bool,
    payload: Vec<u8>,
    header: Option<FrameHeader>,
}

impl DataPacketBuilder {
    /**Initial state is unreliable, mid-frame, and empty payload.*/
    pub fn new(sequence_number: u64)->DataPacketBuilder {
        DataPacketBuilder::with_payload(sequence_number, Vec::default())
    }

    /**Makes a packet with the specified payload, no header, and all flags cleared.*/
    pub fn with_payload(sequence_number: u64, payload: Vec<u8>)->DataPacketBuilder {
        DataPacketBuilder::with_payload_and_header(sequence_number, payload, None)
    }

    /**Configures the builder for mid-frame and using the specified header.

If a header is provided, the packet automatically has its first flag set.*/
    pub fn with_payload_and_header(sequence_number: u64, payload: Vec<u8>, header: Option<FrameHeader>)->DataPacketBuilder {
        DataPacketBuilder {
            sequence_number: sequence_number,
            is_reliable: false,
            is_frame_start: header.is_some(),
            is_frame_end: false,
            payload: payload,
            header: header,
        }
    }

    pub fn set_payload(mut self, payload: Vec<u8>)->Self {
        self.payload = payload;
        self
    }

    pub fn set_header(mut self, header: Option<FrameHeader>)->Self {
        self.header = header;
        self.is_reliable = header.is_some();
        self
    }

    pub fn set_reliable(mut self, reliable: bool)->Self {
        self.is_reliable = reliable;
        self
    }

    pub fn set_frame_start(mut self, start: bool)->Self {
        self.is_frame_start = start;
        self
    }

    pub fn set_frame_end(mut self, end: bool)->Self {
        self.is_frame_end = end;
        self
    }

    pub fn set_sequence_number(mut self, sequence_number: u64)->Self {
        self.sequence_number = sequence_number;
        self
    }

    /**Panics if the packet is invalid. Building invalid packets is a bug.*/
    pub fn build(self)->DataPacket {
        if self.is_frame_start != self.header.is_some() {
            panic!("Header and start flag mismatch. Start flag = {:?}, header = {:?}", self.is_frame_start, self.header);
        }
        let start_flag  = (self.is_frame_start as u8) << DATA_FRAME_START_BIT;
        let end_flag = (self.is_frame_end as u8) << DATA_FRAME_END_BIT;
        let reliable_flag = (self.is_reliable as u8) << DATA_RELIABLE_BIT;
        let flags = start_flag | end_flag | reliable_flag;
        DataPacket {
            sequence_number: self.sequence_number,
            flags: flags,
            payload: self.payload,
            header: self.header,
        }
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

    pub fn borrow_header(&self)->Option<&FrameHeader> {
        self.header.as_ref()
    }

    pub fn get_header(&self)->Option<FrameHeader> {
        self.header
    }

    pub fn borrow_payload(&self)->&Vec<u8> {
        &self.payload
    }

    pub fn into_payload(self)->Vec<u8> {
        self.payload
    }
}

impl FrameHeader {
    pub fn new(last_reliable_frame: u64, length: u32)->FrameHeader {
        FrameHeader{last_reliable_frame: last_reliable_frame, length: length}
    }
}
