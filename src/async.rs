use std::{result, io, net};
use server;
///Represents a Fastnet error.
#[derive(Debug)]
pub enum Error {
    TimedOut,
    HostNotFound,
    PeerNotFound,
    NotListening,
    IncompatibleVersions,
    ConnectionAborted,
    MessageTooLarge,
    IoError(io::Error),
}

pub type Result<T> = result::Result<T, Error>;

/**A Fastnet server.

Fastnet does not distinguish between clients and servers.  This is used both for connecting to other peers and listening for incoming connections.*/
pub struct Server<H: Handler+Send+'static> {
    server: server::MioServer<H>,
}

impl<H: Handler+Send+'static> Server<H> {
    pub fn new(addr: net::SocketAddr, handler: H)->Result<Server<H>> {
        let s = try!(server::MioServer::new(addr, handler).map_err(Error::IoError));
        Ok(Server{server: s})
    }

    /**Schedule a connection request.
    
    This will cause the associated handler to be passed the result with the specified request ID.*/
    pub fn connect(&mut self, addr: net::SocketAddr, request_id: u64) {
        self.server.with(move |s| s.connect(addr, request_id));
    }

    /**Disconnect from a peer with the specified ID.*/
    pub fn disconnect(&mut self, id: u64, request_id: u64) {
        self.server.with(move |s| s.disconnect(id, request_id));
    }
}

/**An event handler.

The methods in this trait are called in a thread which is running in the background, not on the main thread.  None of them should ever block.*/
pub trait Handler {
    fn connected(&mut self, id: u64, request_id: Option<u64>) {
    }

    fn disconnected(&mut self, id: u64, request_id: Option<u64>) {
    }

    fn incoming_message(&mut self, id: u64, channel: u16, payload: &[u8]) {
    }

    fn request_failed(&mut self, request_id: u64, error: Error) {
    }
}
