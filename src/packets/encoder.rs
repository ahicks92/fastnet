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
                try!(CONNECTION_CHANNEL.encode(destination));
                try!(0u8.encode(destination));
                try!(req.encode(destination));
            },
            Packet::StatusResponse(ref resp) => {
                try!(CONNECTION_CHANNEL.encode(destination));
                try!(1u8.encode(destination));
                try!(resp.encode(destination));
            },
            Packet::Connect => {
                try!(CONNECTION_CHANNEL.encode(destination));
                try!(2u8.encode(destination));
            },
            Packet::Connected(id) => {
                try!(CONNECTION_CHANNEL.encode(destination));
                try!(3u8.encode(destination));
                try!(id.encode(destination));
            },
            Packet::Aborted(ref msg) => {
                try!(CONNECTION_CHANNEL.encode(destination));
                try!(4.encode(destination));
                try!(msg.encode(destination));
            },
            Packet::Heartbeat(value) => {
                try!(HEARTBEAT_CHANNEL.encode(destination));
                try!(value.encode(destination));
            },
            Packet::ResetMTUCount{channel: chan} => {
                if chan != MTU_CLIENT_ESTIMATION_CHANNEL || chan != MTU_SERVER_ESTIMATION_CHANNEL {return Err(Invalid)};
                try!(chan.encode(destination));
                try!(0u8.encode(destination));
            },
            Packet::MTUCountWasReset{channel: chan} => {
                if chan != MTU_CLIENT_ESTIMATION_CHANNEL || chan != MTU_SERVER_ESTIMATION_CHANNEL {return Err(Invalid)};
                try!(chan.encode(destination));
                try!(1u8.encode(destination));
            },
            Packet::MTUEstimate{channel: chan, payload: ref p} => {
                if chan != MTU_CLIENT_ESTIMATION_CHANNEL || chan != MTU_SERVER_ESTIMATION_CHANNEL {return Err(Invalid)};
                try!(chan.encode(destination));
                try!(2u8.encode(destination));
                try!(destination.write_all(p).or(Err(TooLarge)));
            },
            Packet::MTUResponse{channel: chan, count: c} => {
                if chan != MTU_CLIENT_ESTIMATION_CHANNEL || chan != MTU_SERVER_ESTIMATION_CHANNEL {return Err(Invalid)};
                try!(chan.encode(destination));
                try!(3u8.encode(destination));
                try!(c.encode(destination));
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
                try!(0u8.encode(destination));
            },
            StatusRequest::VersionQuery => {
                try!(1u8.encode(destination));
            },
            StatusRequest::ExtensionQuery(ref name) => {
                try!(2u8.encode(destination));
                try!(name.encode(destination));
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
                try!(0u8.encode(destination));
                try!(value.encode(destination));
            },
            StatusResponse::VersionResponse(ref version) => {
                try!(1u8.encode(destination));
                try!(version.encode(destination));
            },
            StatusResponse::ExtensionResponse{ref name, supported} => {
                try!(2u8.encode(destination));
                try!(name.encode(destination));
                try!(supported.encode(destination));
            },
        }
        Ok(())
    }
}

//Encoding primitive types:

impl Encodable for bool {
    fn encode(&self, destination: &mut PacketWriter)->Result<(), PacketEncodingError> {
        if *self {1u8.encode(destination)}
        else {0u8.encode(destination)}
    }
}
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
    try!(data.encode(destination));
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

//We can write an impl for slices of encodable things, if we want.

impl<T> Encodable for [T] where T: Encodable {
    fn encode(&self, destination: &mut PacketWriter)->Result<(), PacketEncodingError> {
        for i in self {
            try!(i.encode(destination));
        }
        Ok(())
    }
}
