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
    //Not enough space in the buffer.
    TooLarge,
    //Data didn't validate.
    Invalid,
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

//This overrides the buffer even if it fails.
pub fn encode_packet(packet: &Packet, destination: &mut [u8])->Result<usize, PacketEncodingError> {
    match *packet {
        Packet::StatusRequest(ref req) => {
            return encode_status_request(req, destination);
        },
        Packet::StatusResponse(ref resp) => {
            return encode_status_response(resp, destination);
        },
        Packet::Connect => {
            return encode_connect(destination);
        },
        Packet::Connected(id) => {
            return encode_connected(id, destination);
        },
        Packet::Aborted(ref msg) => {
            return encode_aborted(msg, destination);
        },
        Packet::Heartbeat(value) => {
            return encode_heartbeat(value, destination);
        },
        Packet::ResetMTUCount{channel: chan} => {
            return encode_reset_mtu_count(chan, destination);
        },
        Packet::MTUCountWasReset{channel: chan} => {
            return encode_mtu_count_was_reset(chan, destination);
        },
        Packet::MTUEstimate{channel: chan, payload: ref p} => {
            return encode_mtu_estimate(chan, p, destination);
        },
        Packet::MTUResponse{channel: chan, count: c} => {
            return encode_mtu_response(chan, c, destination);
        },
    }
}

fn encode_status_request(req: &StatusRequest, mut destination: &mut[u8])-> Result<usize, PacketEncodingError> {
    use self::PacketEncodingError::*;
    let initial_count = destination.len();
    try!(destination.write_i16::<BigEndian>(CONNECTION_CHANNEL).or(Err(TooLarge)));
    match *req {
        StatusRequest::FastnetQuery => {
            try!(destination.write_u8(0).or(Err(TooLarge)));
        },
        StatusRequest::VersionQuery => {
            try!(destination.write_u8(1).or(Err(TooLarge)));
        },
        StatusRequest::ExtensionQuery(ref name) => {
            try!(destination.write_u8(2).or(Err(TooLarge)));
            try!(destination.write_all(name.as_bytes()).or(Err(TooLarge)));
        },
    }
    Ok(initial_count-destination.len())
}

fn encode_status_response(resp: &StatusResponse, mut destination: &mut[u8])->Result<usize, PacketEncodingError> {
    use self::PacketEncodingError::*;
    let initial_count = destination.len();
    try!(destination.write_i16::<BigEndian>(CONNECTION_CHANNEL).or(Err(TooLarge)));
    try!(destination.write_u8(1).or(Err(TooLarge)));
    match* resp {
        StatusResponse::FastnetResponse(value) => {
            try!(destination.write_u8(0).or(Err(TooLarge)));
            try!(destination.write_u8(value).or(Err(TooLarge)));
        },
        StatusResponse::VersionResponse(ref version) => {
            try!(destination.write_u8(1).or(Err(TooLarge)));
            try!(destination.write_all(b"1.0").or(Err(TooLarge)));
        },
        StatusResponse::ExtensionResponse{name: ref name, supported: supported} => {
            try!(destination.write_u8(2).or(Err(TooLarge)));
            try!(destination.write(name.as_bytes()).or(Err(TooLarge)));
            try!(destination.write_u8(supported).or(Err(TooLarge)));
        },
    }
    return Ok(initial_count-destination.len());
}

fn encode_connect(mut destination: &mut[u8])->Result<usize, PacketEncodingError> {
    use self::PacketEncodingError::*;
    let initial_count = destination.len();
    try!(destination.write_i16::<BigEndian>(CONNECTION_CHANNEL).or(Err(TooLarge)));
    try!(destination.write_u8(2).or(Err(TooLarge)));
    Ok(initial_count-destination.len())
}

fn encode_connected(id: u32, mut destination: &mut[u8])->Result<usize, PacketEncodingError> {
    use self::PacketEncodingError::*;
    let initial_count = destination.len();
    try!(destination.write_i16::<BigEndian>(CONNECTION_CHANNEL).or(Err(TooLarge)));
    try!(destination.write_u8(3).or(Err(TooLarge)));
    try!(destination.write_u32::<BigEndian>(id).or(Err(TooLarge)));
    Ok(initial_count-destination.len())
}

fn encode_aborted(msg: &String, mut destination: &mut[u8])->Result<usize, PacketEncodingError> {
    use self::PacketEncodingError::*;
    let initial_count = destination.len();
    try!(destination.write_i16::<BigEndian>(CONNECTION_CHANNEL).or(Err(TooLarge)));
    try!(destination.write_u8(4).or(Err(TooLarge)));
    try!(destination.write_all(msg.as_bytes()).or(Err(TooLarge)));;
    Ok(initial_count-destination.len())
}

fn encode_heartbeat(value: i16, mut destination: &mut[u8])->Result<usize, PacketEncodingError> {
    use self::PacketEncodingError::*;
    let initial_count = destination.len();
    try!(destination.write_i16::<BigEndian>(HEARTBEAT_CHANNEL).or(Err(TooLarge)));
    try!(destination.write_i16::<BigEndian>(value).or(Err(TooLarge)));
    Ok(initial_count-destination.len())
}

fn encode_reset_mtu_count(chan: i16, mut destination: &mut[u8])->Result<usize, PacketEncodingError> {
    use self::PacketEncodingError::*;
    if chan != CLIENT_MTU_ESTIMATION_CHANNEL || chan != SERVER_MTU_ESTIMATION_CHANNEL {return Err(Invalid)};
    let initial_count = destination.len();
    try!(destination.write_i16::<BigEndian>(chan).or(Err(TooLarge)));
    try!(destination.write_u8(0).or(Err(TooLarge)));
    Ok(initial_count-destination.len())
}

fn encode_mtu_count_was_reset(chan: i16, mut destination: &mut[u8])->Result<usize, PacketEncodingError> {
    use self::PacketEncodingError::*;
    if chan != CLIENT_MTU_ESTIMATION_CHANNEL || chan != SERVER_MTU_ESTIMATION_CHANNEL {return Err(Invalid)};
    let initial_count = destination.len();
    try!(destination.write_i16::<BigEndian>(chan).or(Err(TooLarge)));
    try!(destination.write_u8(1).or(Err(TooLarge)));
    Ok(initial_count-destination.len())
}

fn encode_mtu_estimate(chan: i16, payload: &[u8], mut destination: &mut[u8])->Result<usize, PacketEncodingError> {
    use self::PacketEncodingError::*;
    if chan != CLIENT_MTU_ESTIMATION_CHANNEL || chan != SERVER_MTU_ESTIMATION_CHANNEL {return Err(Invalid)};
    let initial_count = destination.len();
    try!(destination.write_i16::<BigEndian>(chan).or(Err(TooLarge)));
    try!(destination.write_u8(2).or(Err(TooLarge)));
    try!(destination.write_all(payload).or(Err(TooLarge)));
    Ok(initial_count-destination.len())
}

fn encode_mtu_response(chan: i16, count: u32, mut destination: &mut[u8])->Result<usize, PacketEncodingError> {
    use self::PacketEncodingError::*;
    let initial_count = destination.len();
    try!(destination.write_i16::<BigEndian>(chan).or(Err(TooLarge)));
    try!(destination.write_u8(3).or(Err(TooLarge)));
    try!(destination.write_u32::<BigEndian>(count).or(Err(TooLarge)));
    Ok(initial_count-destination.len())
}
