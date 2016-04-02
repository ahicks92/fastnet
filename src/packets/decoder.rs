use super::*;
use std::io::{self, Read};
use std::cmp;
use byteorder::{BigEndian, ReadBytesExt};

#[derive(Debug)]
pub enum PacketDecodingError {
    //We need more bytes than what we got.
    TooSmall,
    Invalid,
}

pub struct PacketReader<'a> {
    slice: &'a [u8],
    index: usize,
}

impl<'a> PacketReader<'a> {
    pub fn new(destination: &[u8])->PacketReader {
        PacketReader{slice: destination, index: 0}
    }

    pub fn available(&self)->usize {
        self.slice.len()-self.index
    }

    pub fn read_count(&self)->usize {
        self.index
    }
}

impl<'a> Read for PacketReader<'a> {
    fn read(&mut self, buf: &mut[u8])->io::Result<usize> {
        let will_read = cmp::min(buf.len(), self.available());
        for i in 0..will_read {
            buf[i] = self.slice[self.index+i];
        }
        self.index += will_read;
        Ok(will_read)
    }
}

pub trait Decodable {
    type Output;
    fn decode(source: &mut PacketReader)->Result<Self::Output, PacketDecodingError>;
}

impl Decodable for StatusRequest {
    type Output = StatusRequest;
    fn decode(source: &mut PacketReader)->Result<Self::Output, PacketDecodingError> {
        use self::PacketDecodingError::*;
        use super::StatusRequest::*;
        let code = try!(u8::decode(source));
        match code {
            STATUS_FASTNET_SPECIFIER => {return Ok(FastnetQuery);},
            STATUS_VERSION_SPECIFIER => {return Ok(VersionQuery);},
            STATUS_EXTENSION_SPECIFIER => {
                let extension_name = try!(String::decode(source));
                return Ok(ExtensionQuery(extension_name));
            },
            _ => {return Err(Invalid);},
        }
    }
}

impl Decodable for bool {
    type Output = bool;
    fn decode(source: &mut PacketReader)->Result<bool, PacketDecodingError> {
        let code = try!(u8::decode(source));
        return match code {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(PacketDecodingError::Invalid),
        }
    }
}

impl Decodable for i8 {
    type Output = i8;
    fn decode(source: &mut PacketReader)->Result<Self::Output, PacketDecodingError> {
        Ok(try!(source.read_i8().or(Err(PacketDecodingError::TooSmall))))
    }
}

impl Decodable for u8 {
    type Output = u8;
    fn decode(source: &mut PacketReader)->Result<Self::Output, PacketDecodingError> {
        Ok(try!(source.read_u8().or(Err(PacketDecodingError::TooSmall))))
    }
}

impl Decodable for i16 {
    type Output = i16;
    fn decode(source: &mut PacketReader)->Result<Self::Output, PacketDecodingError> {
        Ok(try!(source.read_i16::<BigEndian>().or(Err(PacketDecodingError::TooSmall))))
    }
}

impl Decodable for u16 {
    type Output = u16;
    fn decode(source: &mut PacketReader)->Result<Self::Output, PacketDecodingError> {
        Ok(try!(source.read_u16::<BigEndian>().or(Err(PacketDecodingError::TooSmall))))
    }
}

impl Decodable for i32 {
    type Output = i32;
    fn decode(source: &mut PacketReader)->Result<Self::Output, PacketDecodingError> {
        Ok(try!(source.read_i32::<BigEndian>().or(Err(PacketDecodingError::TooSmall))))
    }
}

impl Decodable for u32 {
    type Output = u32;
    fn decode(source: &mut PacketReader)->Result<Self::Output, PacketDecodingError> {
        Ok(try!(source.read_u32::<BigEndian>().or(Err(PacketDecodingError::TooSmall))))
    }
}

impl Decodable for i64 {
    type Output = i64;
    fn decode(source: &mut PacketReader)->Result<Self::Output, PacketDecodingError> {
        Ok(try!(source.read_i64::<BigEndian>().or(Err(PacketDecodingError::TooSmall))))
    }
}

impl Decodable for u64 {
    type Output = u64;
    fn decode(source: &mut PacketReader)->Result<Self::Output, PacketDecodingError> {
        Ok(try!(source.read_u64::<BigEndian>().or(Err(PacketDecodingError::TooSmall))))
    }
}

impl Decodable for String {
    type Output = String;
    fn decode(source: &mut PacketReader)->Result<Self::Output, PacketDecodingError> {
        let data = &source.slice[source.index..];
        let mut index_of_null: Option<usize>  = None;
        for i in 0..data.len() {
            if data[i] == 0 {
                index_of_null = Some(i);
                break;
            }
        }
        if let Some(extracted_index) = index_of_null {
            let string_slice = &data[..extracted_index];
            source.index += extracted_index+1; //advance it.
            return String::from_utf8(string_slice.to_vec()).or(Err(PacketDecodingError::Invalid));
        }
        else {
            return Err(PacketDecodingError::Invalid);
        }
    }
}
