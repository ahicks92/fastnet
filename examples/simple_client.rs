extern crate fastnet;
extern crate env_logger;
use std::net::{ToSocketAddrs};

#[derive(Default)]
struct EventHandler;

impl fastnet::Handler for EventHandler {
}

fn main() {
    env_logger::init().unwrap();
    let mut i = "127.0.0.1:10000".to_socket_addrs().unwrap();
    let server_addr = i.next().unwrap();
    i = "0.0.0.0:11000".to_socket_addrs().unwrap();
    let our_addr = i.next().unwrap();
    let maybe_serv = fastnet::Server::new(our_addr, EventHandler::default());
    if let Err(ref what) = maybe_serv {
        format!("Error: {:?}", what);
    }
    let mut serv = maybe_serv.unwrap();
    serv.connect(server_addr, 0);
    println!("Server is running.");
    loop {}
}
