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
    let addr = i.next().unwrap();
    let maybe_serv = fastnet::Server::new(addr, fastnet::PrintingHandler::new());
    if let Err(ref what) = maybe_serv {
        format!("Error: {:?}", what);
    }
    let mut serv = maybe_serv.unwrap();
    println!("Server is running.");
    loop {}
}
