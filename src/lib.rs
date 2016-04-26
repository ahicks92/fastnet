/*! The main Fastnet module.

This is the low-level API.  If your goal is extremely high-performance usage, this is the API you want.  See the blocking module for a simpler API which is less annoying for common use cases.*/
#![allow(warnings)]

extern crate byteorder;
extern crate mio;
extern crate crc;

mod packets;
mod server;
mod status_translator;
mod async;

pub use async::*;
