/*! The main Fastnet module.

This is the low-level API.  If your goal is extremely high-performance usage, this is the API you want.  See the blocking module for a simpler API which is less annoying for common use cases.*/
#![allow(warnings)]

extern crate byteorder;
extern crate mio;
extern crate crc;

mod packets;
mod server;
mod status_translator;

use std::{result, io, net};

///Represents a Fastnet error.
#[derive(Debug, Copy, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub enum Error {
    TimedOut,
    HostNotFound,
    PeerNotFound,
    MessageTooLarge,
}

pub type Result<T> = result::Result<T, Error>;

/**A Fastnet server.

Fastnet does not distinguish between clients and servers.  This is used both for connecting to other peers and listening for incoming connections.*/
#[derive(Default)]
pub struct Server;

impl Server {
    pub fn new<H: Handler+Send+'static>(addr: net::SocketAddr, handler: H)->Result<Server> {
        Ok(Server::default())
    }

    /**For debugging, as this implementation is currently alpha.
    
    This will go away, but enables printing sent/received packets to stdout, among other things.*/
    pub fn enable_debug_prints(&mut self) {

    /**Schedule a connection request.
    
    This will cause the associated handler to be passed the result with the specified request ID.*/
    pub fn connect(addr: net::SocketAddr, request_id: u64) {
    }

    /**Disconnect from a peer with the specified ID.*/
    pub fn disconnect(id: u64, request_id: u64) {
    }
}

/**An event handler.

The methods in this trait are called in a thread which is running in the background, not on the main thread.  None of them should ever block.*/
pub trait Handler {
    fn connected(&mut self, id: u64, request_id: Option<u64>);
    fn disconnected(&mut self, id: u64, request_id: Option<u64>);
    fn message(&mut self, id: u64, channel: u16, payload: &[u8]);
    fn request_failed(&mut self, request_id: u64, error: Error);
}
