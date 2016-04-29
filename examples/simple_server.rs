extern crate fastnet;
extern crate env_logger;
use std::net::{ToSocketAddrs};

fn main() {
    env_logger::init().unwrap();
    let mut i = "0.0.0.0:10000".to_socket_addrs().unwrap();
    let addr = i.next().unwrap();
    let maybe_serv = fastnet::Server::new(addr, fastnet::PrintingHandler::new());
    if let Err(ref what) = maybe_serv {
        format!("Error: {:?}", what);
    }
    let mut serv = maybe_serv.unwrap();
    println!("Server is running.");
    loop {}
}
