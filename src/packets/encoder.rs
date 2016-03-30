use super::*;
use byteorder::{BigEndian, WriteBytesExt};
use std::io::{self, Write};
use std::cmp;

#[derive(Debug)]
pub enum PacketEncodingError {
    //Not enough space in the buffer.
    TooLarge,
    //Data didn't validate.
    Invalid,
}

pub struct PacketWriter<'a> {
    slice: &'a mut[u8],
    index: usize,
}

impl<'a> PacketWriter<'a> {
    pub fn new(destination: &'a mut[u8])->PacketWriter {
        PacketWriter{slice: destination, index: 0}
    }

    pub fn written(&self)->usize {
        self.index
    }

    pub fn available(&self)->usize {
        self.slice.len()-self.index
    }
}

impl<'a> Write for PacketWriter<'a> {
    fn write(&mut self, buf: &[u8])->io::Result<usize> {
        let available = self.slice.len()-self.index;
        let willWrite = cmp::min(available, buf.len());
        for i in 0..willWrite {
            self.slice[self.index+i] = buf[i];
        }
        self.index += willWrite;
        Ok(willWrite)
    }

    fn flush(&mut self)->io::Result<()> {
        Ok(())
    }
}

pub trait Encodable {
    fn encode(&self, destination: &mut PacketWriter)->Result<(), PacketEncodingError>;
}

impl Encodable for Packet {
    fn encode(&self, destination: &mut PacketWriter)->Result<(), PacketEncodingError> {
        use self::PacketEncodingError::*;
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
    Ok(())
    }
}

impl Encodable for StatusRequest {
    fn encode(&self, destination: &mut PacketWriter)->Result<(), PacketEncodingError> {
        use self::PacketEncodingError::*;
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
        Ok(())
    }
}

impl Encodable for StatusResponse {
    fn encode(&self, destination: &mut PacketWriter)->Result<(), PacketEncodingError> {
        use self::PacketEncodingError::*;
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
        Ok(())
    }
}

//Encoding integer types:

impl Encodable for i8 {
    fn encode(&self, destination: &mut PacketWriter)->Result<(), PacketEncodingError> {
        try!(destination.write_i8(*self).or(Err(PacketEncodingError::TooLarge)));
        Ok(())
    }
}

impl Encodable for u8 {
    fn encode(&self, destination: &mut PacketWriter)->Result<(), PacketEncodingError> {
        try!(destination.write_u8(*self).or(Err(PacketEncodingError::TooLarge)));
        Ok(())
    }
}

impl Encodable for i16 {
    fn encode(&self, destination: &mut PacketWriter)->Result<(), PacketEncodingError> {
        try!(destination.write_i16::<BigEndian>(*self).or(Err(PacketEncodingError::TooLarge)));
        Ok(())
    }
}

impl Encodable for u16 {
    fn encode(&self, destination: &mut PacketWriter)->Result<(), PacketEncodingError> {
        try!(destination.write_u16::<BigEndian>(*self).or(Err(PacketEncodingError::TooLarge)));
        Ok(())
    }
}

impl Encodable for i32 {
    fn encode(&self, destination: &mut PacketWriter)->Result<(), PacketEncodingError> {
        try!(destination.write_i32::<BigEndian>(*self).or(Err(PacketEncodingError::TooLarge)));
        return Ok(())
    }
}

impl Encodable for u32 {
    fn encode(&self, destination: &mut PacketWriter)->Result<(), PacketEncodingError> {
        try!(destination.write_u32::<BigEndian>(*self).or(Err(PacketEncodingError::TooLarge)));
        Ok(())
    }
}

//These are the two string types.
//The Fastnet spec requires null-termination.  Rust strings are not null terminated.

fn encode_string_slice(data: &[u8], destination: &mut PacketWriter)->Result<(), PacketEncodingError> {
    use self::PacketEncodingError::*;
    if data.iter().any(|&x| x == 0) {return Err(Invalid)};
    try!(destination.write_all(data).or(Err(TooLarge)));
    try!(0u8.encode(destination));
    Ok(())
}

impl Encodable for str {
    fn encode(&self, destination: &mut PacketWriter)->Result<(), PacketEncodingError> {
        encode_string_slice(self.as_bytes(), destination)
    }
}

impl Encodable for String {
    fn encode(&self, destination: &mut PacketWriter)->Result<(), PacketEncodingError> {
        encode_string_slice(self.as_bytes(), destination)
    }
}

