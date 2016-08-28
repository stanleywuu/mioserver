use connection;
use bytes::{Buf, RingBuf, SliceBuf, MutBuf, ByteBuf};

use std::io;
use std::io::{Error, ErrorKind};
use std::net::SocketAddr;
use std::str::FromStr;

use mio::*;
use mio::tcp::*;
use mio::util::Slab;

use macros;

use transactionstorage;
use transactionstorage::SqliteDB;
use transactionstorage::Transaction;

pub struct Server {
    // main socket for our server
    sock: TcpListener,

    // token of our server. we keep track of it here instead of doing `const SERVER = Token(0)`.
    token: Token,
    
    // a list of connections _accepted_ by our server
    conns: Slab<connection::Connection>,

	db: transactionstorage::SqliteDB
}

impl Handler for Server {
    type Timeout = u32;
    type Message = ();

	fn timeout(&mut self, event_loop: &mut EventLoop<Server>, timeout: Self::Timeout) {
	 // Queue up a write for all connected clients.
        for conn in self.conns.iter_mut() {
        // println!("Tick: {:?}", conn.token);
			conn.handle_heartbeat();
        }
		event_loop.timeout_ms(123, 1000).unwrap();
	}
	
    fn ready(&mut self, event_loop: &mut EventLoop<Server>, token: Token, events: EventSet) {
        info!("events = {:?}", events);
        assert!(token != Token(0), "[BUG]: Received event for Token(0)");

        if events.is_error() {
            println!("Error event for {:?}", token);
            self.reset_connection(event_loop, token);
            return;
        }

        if events.is_hup() {
            info!("Hup event for {:?}", token);
            self.reset_connection(event_loop, token);
            return;
        }

        // We never expect a write event for our `Server` token . A write event for any other token
        // should be handed off to that connection.
        if events.is_writable() {
            info!("Write event for {:?}", token);
            assert!(self.token != token, "Received writable event for Server");

            self.find_connection_by_token(token).writable()
                .and_then(|_| self.find_connection_by_token(token).reregister(event_loop))
                .unwrap_or_else(|e| {
                    println!("Write event failed for {:?}, {:?}", token, e);
                    self.reset_connection(event_loop, token);
                });
        }

        // A read event for our `Server` token means we are establishing a new connection. A read
        // event for any other token should be handed off to that connection.
        if events.is_readable() {
            info!("Read event for {:?}", token);
            if self.token == token {
                self.accept(event_loop);
            } else {

                self.readable(event_loop, token)
                    .and_then(|_| self.find_connection_by_token(token).reregister(event_loop))
                    .unwrap_or_else(|e| {
                        println!("Read event failed for {:?}: {:?}", token, e);
                        self.reset_connection(event_loop, token);
                    });
            }
        }
    }
}

impl Server {
    pub fn new(sock: TcpListener) -> Server {
		let sqlite = transactionstorage::SqliteDB::new("db/messages.db");
		let dbCopy = sqlite.clone();
		sqlite.createDB();
		
        Server {
            sock: sock,

            // I don't use Token(0) because kqueue will send stuff to Token(0)
            // by default causing really strange behavior. This way, if I see
            // something as Token(0), I know there are kqueue shenanigans
            // going on.
            token: Token(1),
			
			db: dbCopy,

            // SERVER is Token(1), so start after that
            // we can deal with a max of 126 connections
            conns: Slab::new_starting_at(Token(2), 128)
        }
    }

    /// Register Server with the event loop.
    ///
    /// This keeps the registration details neatly tucked away inside of our implementation.
    pub fn register(&mut self, event_loop: &mut EventLoop<Server>) -> io::Result<()> {
		event_loop.timeout_ms(123, 1000).unwrap();
        event_loop.register(
            &self.sock,
            self.token,
            EventSet::readable(),
            PollOpt::edge() | PollOpt::oneshot()
        ).or_else(|e| {
            println!("Failed to register server {:?}, {:?}", self.token, e);
            Err(e)
        })
    }

    /// Register Server with the event loop.
    ///
    /// This keeps the registration details neatly tucked away inside of our implementation.
    fn reregister(&mut self, event_loop: &mut EventLoop<Server>) {
        event_loop.reregister(
            &self.sock,
            self.token,
            EventSet::readable(),
            PollOpt::edge() | PollOpt::oneshot()
        ).unwrap_or_else(|e| {
            println!("Failed to reregister server {:?}, {:?}", self.token, e);
            let server_token = self.token;
            self.reset_connection(event_loop, server_token);
        })
    }

    /// Accept a _new_ client connection.
    ///
    /// The server will keep track of the new connection and forward any events from the event loop
    /// to this connection.
    fn accept(&mut self, event_loop: &mut EventLoop<Server>) {
        info!("server accepting new socket");

        // Log an error if there is no socket, but otherwise move on so we do not tear down the
        // entire server.
        let sock = match self.sock.accept() {
            Ok(s) => {
                match s {
                    Some(sock) => sock.0,
                    None => {
                        println!("Failed to accept new socket");
                        self.reregister(event_loop);
                        return;
                    }
                }
            },
            Err(e) => {
                println!("Failed to accept new socket, {:?}", e);
                self.reregister(event_loop);
                return;
            }
        };
		
		let db = self.db.clone();

        // `Slab#insert_with` is a wrapper around `Slab#insert`. I like `#insert_with` because I
        // make the `Token` required for creating a new connection.
        //
        // `Slab#insert` returns the index where the connection was inserted. Remember that in mio,
        // the Slab is actually defined as `pub type Slab<T> = ::slab::Slab<T, ::Token>;`. Token is
        // just a tuple struct around `usize` and Token implemented `::slab::Index` trait. So,
        // every insert into the connection slab will return a new token needed to register with
        // the event loop. Fancy...
        match self.conns.insert_with(|token| {
            println!("registering {:?} with event loop", token);
            connection::Connection::new(sock, token, db)
        }) {
            Some(token) => {
                // If we successfully insert, then register our connection.
                match self.find_connection_by_token(token).register(event_loop) {
                    Ok(_) => {
						self.find_connection_by_token(token).welcome();
					},
                    Err(e) => {
                        println!("Failed to register {:?} connection with event loop, {:?}", token, e);
                        self.conns.remove(token);
                    }
                }
            },
            None => {
                // If we fail to insert, `conn` will go out of scope and be dropped.
                println!("Failed to insert connection into slab");
            }
        };

        // We are using edge-triggered polling. Even our SERVER token needs to reregister.
        self.reregister(event_loop);
    }

    /// Forward a readable event to an established connection.
    ///
    /// Connections are identified by the token provided to us from the event loop. Once a read has
    /// finished, push the receive buffer into the all the existing connections so we can
    /// broadcast.
    fn readable(&mut self, event_loop: &mut EventLoop<Server>, token: Token) -> io::Result<()> {
        info!("server conn readable; token={:?}", token);
        let message = try!(self.find_connection_by_token(token).readable());

        if message.remaining() == message.capacity() { // is_empty
            return Ok(());
        }

        // TODO pipeine this whole thing
        let mut bad_tokens = Vec::new();
		
        // Queue up a write for all connected clients.
        for conn in self.conns.iter_mut() {
			if conn.token == token
			{
            // TODO: use references so we don't have to clone
			let bytes = message.bytes();
            let conn_send_buf = ByteBuf::from_slice(bytes);
            conn.handle_input(&message)
                .and_then(|_| conn.reregister(event_loop))
                .unwrap_or_else(|e| {
                    println!("Failed to queue message for {:?}: {:?}", conn.token, e);
                    // We have a mutable borrow for the connection, so we cannot remove until the
                    // loop is finished
                    bad_tokens.push(conn.token)
                });
			}
        }

        for t in bad_tokens {
            self.reset_connection(event_loop, t);
        }

        Ok(())
    }

    fn reset_connection(&mut self, event_loop: &mut EventLoop<Server>, token: Token) {
        if self.token == token {
            event_loop.shutdown();
        } else {
            println!("reset connection; token={:?}", token);
            self.conns.remove(token);
        }
    }

    /// Find a connection in the slab using the given token.
    fn find_connection_by_token<'a>(&'a mut self, token: Token) -> &'a mut connection::Connection {
        &mut self.conns[token]
    }
}