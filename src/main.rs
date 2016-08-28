extern crate mio;
extern crate bytes;

//#[macro_use] extern crate log;

extern crate env_logger;

use std::net::SocketAddr;
use std::str::FromStr;

use mio::*;
use mio::tcp::*;
use mio::util::Slab;

#[macro_use]mod macros;
mod connection;
mod server;
mod Messages;
mod character;
mod logon;
mod gamehandler;
mod storage;
mod transactionstorage;

macro_rules! info {
    ($fmt:expr) => (print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => (print!(concat!($fmt, "\n"), $($arg)*));
}

macro_rules! error {
    ($fmt:expr) => (print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => (print!(concat!($fmt, "\n"), $($arg)*));
}



fn main() {

    // Before doing anything, let us register a logger. The mio library has really good logging
    // at the _println_ and _debug_ levels. Having a logger setup is invaluable when trying to
    // figure out why something is not working correctly.
    env_logger::init().ok().expect("Failed to init logger");

    let addr: SocketAddr = FromStr::from_str("127.0.0.1:8000")
        .ok().expect("Failed to parse host:port string");
    let sock = TcpListener::bind(&addr).ok().expect("Failed to bind address");

    let mut event_loop = EventLoop::new().ok().expect("Failed to create event loop");
	
    // Create our Server object and register that with the event loop. I am hiding away
    // the details of how registering works inside of the `Server#register` function. One reason I
    // really like this is to get around having to have `const SERVER = Token(0)` at the top of my
    // file. It also keeps our polling options inside `Server`.
    let mut server = server::Server::new(sock);
    server.register(&mut event_loop).ok().expect("Failed to register server with event loop");

    info!("Even loop starting...");
    event_loop.run(&mut server).ok().expect("Failed to start event loop");
}