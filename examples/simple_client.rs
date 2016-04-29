extern crate fastnet;
extern crate env_logger;
use std::net::{ToSocketAddrs};
use std::env;

fn main() {
    env_logger::init().unwrap();
    let args = env::args().collect::<Vec<_>>();
    if args.len() != 2{
        println!("Syntax: {} <address>", args[0]);
        return;
    }
    let mut i = args[1].to_socket_addrs().unwrap();
    let server_addr = i.next().unwrap();
    i = "0.0.0.0:11000".to_socket_addrs().unwrap();
    let our_addr = i.next().unwrap();
    let maybe_serv = fastnet::Server::new(our_addr, fastnet::PrintingHandler::new());
    if let Err(ref what) = maybe_serv {
        format!("Error: {:?}", what);
    }
    let mut serv = maybe_serv.unwrap();
    serv.connect(server_addr, 0);
    println!("Server is running.");
    loop {}
}
