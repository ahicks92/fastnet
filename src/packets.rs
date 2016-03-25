use byteorder::{BigEndian, ByteOrder, ReadBytesExt, WriteBytesExt};
use std::io::{Write};

pub enum Packet {
    //Status request and response (channel -1)
    StatusRequest(StatusRequest),
    StatusResponse(StatusResponse),

    //Connection handshake (also channel -1).
    Connect,
    Aborted(String),
    Connected(u32),
    
    //Heartbeat (channel -2).
    Heartbeat(i16),
    
    //MTU estimation (-3 and -4)
    //These have to record the channel.
    ResetMTUCount {channel: i16},
    MTUCountWasReset {channel: i16},
    MTUEstimate {channel: i16, payload: Vec<u8>},
    MTUResponse {channel: i16, count: u32},
}

pub enum StatusRequest {
    FastnetQuery,
    VersionQuery,
    ExtensionQuery(String),
}

pub enum StatusResponse {
    FastnetResponse(u8),
    VersionResponse(String),
    ExtensionResponse {name: String, supported: u8},
}

pub enum PacketEncodingError {
    //We need to write over 500 bytes to decode.
    TooLarge,
}

pub enum PacketDecodingError {
    //We need more bytes than what we got.
    TooSmall,
    //The packet has a checksum, but we didn't match it.
    ChecksumMismatch,
    UnknownChannel,
    InvalidFormat,
}

//These are the channel constants.

//Status query and connection handshake channel.
const CONNECTION_CHANNEL: i16 = -1;

//Heartbeat channel.
const HEARTBEAT_CHANNEL: i16 = -2;

//Client and server MTU channels.
//These are named for the entity executing the MTU estimation algorithm, not for the entity receiving.
const SERVER_MTU_ESTIMATION_CHANNEL: i16 = -3;
const CLIENT_MTU_ESTIMATION_CHANNEL: i16 = -4;

//Specification mandates that we never send a packet over the maximum size of 500 bytes.
const MAXIMUM_PACKET_SIZE: usize = 500;

//This function writes over the buffer even if it fails.
pub fn encode_packet(packet: &Packet, destination: &mut [u8; MAXIMUM_PACKET_SIZE])->Result<usize, PacketEncodingError> {
    let destinationLen = destination.len();
    let mut writingTo = &mut destination[..];
    match *packet {
        Packet::StatusRequest(ref req) => {
            writingTo.write_i16::<BigEndian>(-1).unwrap();
            writingTo.write_i16::<BigEndian>(0).unwrap();
            match *req {
                StatusRequest::FastnetQuery => {
                    writingTo.write_u8(0).unwrap();
                },
                StatusRequest::VersionQuery => {
                    writingTo.write_u8(1).unwrap();
                },
                StatusRequest::ExtensionQuery(ref name) => {
                    writingTo.write_u8(2).unwrap();
                    let nameAsBytes = name.as_bytes();
                    if nameAsBytes.len() < writingTo.len()-1 {return Err(PacketEncodingError::TooLarge);};
                    writingTo.write(nameAsBytes).unwrap();
                },
            }
        },
        Packet::StatusResponse(ref resp) => {
            writingTo.write_i16::<BigEndian>(-1);
            writingTo.write_u8(1);
            match* resp {
                StatusResponse::FastnetResponse(value) => {
                    writingTo.write_u8(0).unwrap();
                    writingTo.write_u8(value);
                },
                StatusResponse::VersionResponse(ref version) => {
                    writingTo.write_u8(1);
                    let versionAsBytes = version.as_bytes();
                    if writingTo.len() < versionAsBytes.len() {return Err(PacketEncodingError:: TooLarge)};
                    writingTo.write(versionAsBytes).unwrap();
                },
                StatusResponse::ExtensionResponse{name: ref name, supported: supported} => {
                    writingTo.write_u8(2).unwrap();
                    let nameAsBytes = name.as_bytes();
                    //name plus the boolean.
                    if writingTo.len() < nameAsBytes.len()-1 {return Err(PacketEncodingError::TooLarge)};
                    writingTo.write(nameAsBytes).unwrap();
                    writingTo.write_u8(supported);
                },
            }
        },
        Packet::Connect => {
            writingTo.write_i16::<BigEndian>(-1);
            writingTo.write_u8(2);
        },
        Packet::Connected(id) => {
            writingTo.write_i16::<BigEndian>(-1);
            writingTo.write_u8(3);
            writingTo.write_u32::<BigEndian>(id);
        },
        Packet::Aborted(ref msg) => {
            writingTo.write_i16::<BigEndian>(-1);
            writingTo.write_u8(4);
            let msgAsBytes = msg.as_bytes();
            if writingTo.len() < msgAsBytes.len() {return Err(PacketEncodingError::TooLarge)};
            writingTo.write(msgAsBytes).unwrap();
        },
        Packet::Heartbeat(value) => {
            writingTo.write_i16::<BigEndian>(-2);
            writingTo.write_i16::<BigEndian>(value);
        },
        Packet::ResetMTUCount{channel: chan} => {
            writingTo.write_i16::<BigEndian>(chan);
            writingTo.write_u8(0);
        },
        Packet::MTUCountWasReset{channel: chan} => {
            writingTo.write_i16::<BigEndian>(chan).unwrap();
            writingTo.write_u8(1).unwrap();
        },
        Packet::MTUEstimate{channel: chan, payload: ref p} => {
            writingTo.write_i16::<BigEndian>(chan).unwrap();
            writingTo.write_u8(3).unwrap();
            if writingTo.len() < p.len() {return Err(PacketEncodingError::TooLarge)};
            writingTo.write(p).unwrap();
        },
        Packet::MTUResponse{channel: chan, count: c} => {
            writingTo.write_i16::<BigEndian>(chan);
            writingTo.write_u8(4).unwrap();
            writingTo.write_u32::<BigEndian>(c).unwrap();
        },
    }
    Ok(destinationLen-writingTo.len())
}
