extern crate fastnet;
use std::net::{ToSocketAddrs};

#[derive(Default)]
struct EventHandler;

impl fastnet::Handler for EventHandler {
}

fn main() {
    let mut i = "0.0.0.0:10000".to_socket_addrs().unwrap();
    let addr = i.next().unwrap();
    let maybe_serv = fastnet::Server::new(addr, EventHandler::default());
    if let Err(ref what) = maybe_serv {
        format!("Error: {:?}", what);
    }
    let mut serv = maybe_serv.unwrap();
    serv.enable_debug_print();
    println!("Server is running.");
    loop {}
}
