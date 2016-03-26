use super::*;
use byteorder::{BigEndian, WriteBytesExt};
use std::io::Write;

pub enum PacketEncodingError {
    //Not enough space in the buffer.
    TooLarge,
    //Data didn't validate.
    Invalid,
}

pub trait Encodable {
    fn encode(&self, mut destination: &mut[u8])->Result<usize, PacketEncodingError>;
}

impl Encodable for Packet {
    fn encode(&self, mut destination: &mut[u8])->Result<usize, PacketEncodingError> {
        use self::PacketEncodingError::*;
        let initial_count = destination.len();
        match *self {
            Packet::StatusRequest(ref req) => {
                try!(destination.write_i16::<BigEndian>(CONNECTION_CHANNEL).or(Err(TooLarge)));
                try!(destination.write_u8(0).or(Err(TooLarge)));
                try!(req.encode(destination));
            },
            Packet::StatusResponse(ref resp) => {
                try!(destination.write_i16::<BigEndian>(CONNECTION_CHANNEL).or(Err(TooLarge)));
                try!(destination.write_u8(1).or(Err(TooLarge)));
                try!(resp.encode(destination));
            },
            Packet::Connect => {
                try!(destination.write_i16::<BigEndian>(CONNECTION_CHANNEL).or(Err(TooLarge)));
                try!(destination.write_u8(2).or(Err(TooLarge)));
            },
            Packet::Connected(id) => {
                try!(destination.write_i16::<BigEndian>(CONNECTION_CHANNEL).or(Err(TooLarge)));
                try!(destination.write_u8(3).or(Err(TooLarge)));
                try!(destination.write_u32::<BigEndian>(id).or(Err(TooLarge)));
            },
            Packet::Aborted(ref msg) => {
                try!(destination.write_i16::<BigEndian>(CONNECTION_CHANNEL).or(Err(TooLarge)));
                try!(destination.write_u8(4).or(Err(TooLarge)));
                try!(destination.write_all(msg.as_bytes()).or(Err(TooLarge)));
            },
            Packet::Heartbeat(value) => {
                try!(destination.write_i16::<BigEndian>(HEARTBEAT_CHANNEL).or(Err(TooLarge)));
                try!(destination.write_i16::<BigEndian>(value).or(Err(TooLarge)));
            },
            Packet::ResetMTUCount{channel: chan} => {
                if chan != MTU_CLIENT_ESTIMATION_CHANNEL || chan != MTU_SERVER_ESTIMATION_CHANNEL {return Err(Invalid)};
                try!(destination.write_i16::<BigEndian>(chan).or(Err(TooLarge)));
                try!(destination.write_u8(0).or(Err(TooLarge)));
            },
            Packet::MTUCountWasReset{channel: chan} => {
                if chan != MTU_CLIENT_ESTIMATION_CHANNEL || chan != MTU_SERVER_ESTIMATION_CHANNEL {return Err(Invalid)};
                try!(destination.write_i16::<BigEndian>(chan).or(Err(TooLarge)));
                try!(destination.write_u8(1).or(Err(TooLarge)));
            },
            Packet::MTUEstimate{channel: chan, payload: ref p} => {
                if chan != MTU_CLIENT_ESTIMATION_CHANNEL || chan != MTU_SERVER_ESTIMATION_CHANNEL {return Err(Invalid)};
                try!(destination.write_i16::<BigEndian>(chan).or(Err(TooLarge)));
                try!(destination.write_u8(2).or(Err(TooLarge)));
                try!(destination.write_all(p).or(Err(TooLarge)));
            },
            Packet::MTUResponse{channel: chan, count: c} => {
                if chan != MTU_CLIENT_ESTIMATION_CHANNEL || chan != MTU_SERVER_ESTIMATION_CHANNEL {return Err(Invalid)};
                try!(destination.write_i16::<BigEndian>(chan).or(Err(TooLarge)));
                try!(destination.write_u8(3).or(Err(TooLarge)));
                try!(destination.write_u32::<BigEndian>(c).or(Err(TooLarge)));
            },
        }
    Ok(initial_count-destination.len())
    }
}

impl Encodable for StatusRequest {
    fn encode(&self, mut destination: &mut[u8])->Result<usize, PacketEncodingError> {
        use self::PacketEncodingError::*;
        let initial_count = destination.len();
        match *self {
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
}

impl Encodable for StatusResponse {
    fn encode(&self, mut destination: &mut[u8])->Result<usize, PacketEncodingError> {
        use self::PacketEncodingError::*;
        let initial_count = destination.len();
        match *self {
            StatusResponse::FastnetResponse(value) => {
                try!(destination.write_u8(0).or(Err(TooLarge)));
                try!(destination.write_u8(value).or(Err(TooLarge)));
            },
            StatusResponse::VersionResponse(ref version) => {
                try!(destination.write_u8(1).or(Err(TooLarge)));
                try!(destination.write_all(version.as_bytes()).or(Err(TooLarge)));
            },
            StatusResponse::ExtensionResponse{ref name, supported} => {
                try!(destination.write_u8(2).or(Err(TooLarge)));
                try!(destination.write(name.as_bytes()).or(Err(TooLarge)));
                try!(destination.write_u8(supported).or(Err(TooLarge)));
            },
        }
        Ok(initial_count-destination.len())
    }
}
