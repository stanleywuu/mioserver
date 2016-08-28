extern crate time;

use bytes::{Buf, ByteBuf, ByteStr};

use logon;
use logon::LogonState;

use character;
use character::CharCreator;
use character::CreationState;

use transactionstorage;
use transactionstorage::SqliteDB;
use transactionstorage::Transaction;

use gamehandler;
use gamehandler::GameHandler;

use std::io;
use std::io::{Error, ErrorKind};
use std::net::SocketAddr;
use Messages::greeting;

use mio::*;
use mio::tcp::*;

use server;


enum ConnectionState
{
	New,
	Logon,
	CharacterCreation,
	Play,
}

fn get_input_string(message: &ByteBuf) -> String
{
	let input_bytes = message.bytes();
	let mut u8_buf = Vec::new();
	
	for index in 0..input_bytes.len()
	{
		u8_buf.push(input_bytes[index]);
	}
	
	let input_string = String::from_utf8(u8_buf).unwrap();
	input_string
}

/// A stateful wrapper around a non-blocking stream. This connection is not
/// the SERVER connection. This connection represents the client connections
/// _accepted_ by the SERVER connection.
pub struct Connection {
    // handle to the accepted socket
    sock: TcpStream,

    // token used to register with the event loop
    pub token: Token,
	
	// Last updated time
	pub lastUpdate: time::Timespec,
	
	dbclient: transactionstorage::SqliteDB,

    // set of events we are interested in
    interest: EventSet,

    // messages waiting to be sent out
    send_queue: Vec<ByteBuf>,
	
	state: ConnectionState,
	
	logon_handler: logon::LogonManager,
	character_creator: character::CharCreator,
}

impl Connection {
    pub fn new(sock: TcpStream, token: Token, db:transactionstorage::SqliteDB) -> Connection {
        Connection {
            sock: sock,
            token: token,

            // new connections are only listening for a hang up event when
            // they are first created. we always want to make sure we are 
            // listening for the hang up event. we will additionally listen
            // for readable and writable events later on.
            interest: EventSet::hup(),			

            send_queue: Vec::new(),
			
			dbclient: db,
			
			state: ConnectionState::Logon,
			
			lastUpdate: time::get_time(),
			
			logon_handler: logon::LogonManager::new(),
			character_creator: character::CharCreator::new(),			
        }
    }	
	
	pub fn set_last_update(&mut self, updatedTime: time::Timespec)
	{
		self.lastUpdate = updatedTime.clone();
	}
	
	pub fn get_last_update(self) -> time::Timespec
	{
		self.lastUpdate
	}
	
	pub fn handle_heartbeat(&mut self)
	{
		match self.state
		{
			ConnectionState::Play =>
			{
			let mut last_update = self.lastUpdate.clone();
			let dbclient = self.dbclient.clone();
			let messages = dbclient.getRecord(last_update.clone());
			
			for message in messages
			{
				info!("Has message");
				if message.time_created > last_update
				{
					last_update = message.time_created.clone();
					self.send(message.message.clone());
				}
			}
			self.set_last_update(last_update.clone());
			self.writable();
			}
			_=>{}
		}
	}
	
	pub fn handle_input(&mut self, message: &ByteBuf) -> io::Result<bool>
	{
		match self.state
		{
			ConnectionState::New =>
			{
				//greeting
				self.state = ConnectionState::Logon;
				let buf:&[u8] = greeting::WELCOME_MESSAGE.as_bytes();
				self.send_message(ByteBuf::from_slice(&buf)).
				unwrap_or_else(|e|
				{
                        error!("Failed to queue message for {:?}: {:?}", self.token, e);
				});
			}
			ConnectionState::Logon =>
			{
				let input_string = get_input_string(message.clone());
				
				let data_struct = logon::LogonManager::new_from_data(self.logon_handler.username.clone(), self.logon_handler.password.clone(), self.logon_handler.logon_state.clone(), String::new());
				let result = logon::process_commands(input_string, data_struct);								
				self.logon_handler = result;

				let to_send = self.logon_handler.return_msg.clone();
				
				self.send(to_send);
				
				match self.logon_handler.logon_state
				{
					LogonState::RegisterCreation => {self.state = ConnectionState::CharacterCreation;}
					LogonState::Done => {self.state = ConnectionState::Play;}
					_ => {}
				}
			},
			ConnectionState::CharacterCreation =>
			{
				let input_string = get_input_string(message.clone());
				
				// Create character
				let character = 
					character::Character::new_from_data(
						self.logon_handler.username.clone(),
						self.character_creator.character.info.clone(),
						self.character_creator.character.attr.clone());
					
				// Create a copy
				let data_struct = 
					character::CharCreator::new_from_data(
						self.logon_handler.username.clone(),
						character,
						self.character_creator.state.clone(),
						String::new());
				
				// Process commands and send reply
				let result = CharCreator::process_commands(input_string, data_struct);
				self.character_creator = result;
				
				let to_send = self.character_creator.return_msg.clone();				
				self.send(to_send);
				
				match self.character_creator.state
				{
					CreationState::Done => {self.state = ConnectionState::Play;}
					_ => {}
				}
			},
			ConnectionState::Play =>
			{
				let input_string = get_input_string(message.clone());
				
				let character = 
					character::Character::new_from_data(
						self.logon_handler.username.clone(),
						self.character_creator.character.info.clone(),
						self.character_creator.character.attr.clone());
					
				let data_struct = 
					character::CharCreator::new_from_data(
						self.logon_handler.username.clone(),
						character,
						self.character_creator.state.clone(),
						String::new());
				
				let result = GameHandler::process_commands(input_string, data_struct);
				
				let now = time::get_time();
				let transaction = transactionstorage::Transaction::new(result.clone(), now.clone());
				let dbclient = self.dbclient.clone();
				
				dbclient.insertRecord(transaction);
				
				println!("Inserting result in play state {:?}", result);
			}
		}
		Ok(true)
	}

    /// Handle read event from event loop.
    ///
    /// Currently only reads a max of 2048 bytes. Excess bytes are dropped on the floor.
    ///
    /// The recieve buffer is sent back to `Server` so the message can be broadcast to all
    /// listening connections.
    pub fn readable(&mut self) -> io::Result<ByteBuf> {

        // ByteBuf is a heap allocated slice that mio supports internally. We use this as it does
        // the work of tracking how much of our slice has been used. I chose a capacity of 2048
        // after reading 
        // https://github.com/carllerche/mio/blob/eed4855c627892b88f7ca68d3283cbc708a1c2b3/src/io.rs#L23-27
        // as that seems like a good size of streaming. If you are wondering what the difference
        // between messaged based and continuous streaming read
        // http://stackoverflow.com/questions/3017633/difference-between-message-oriented-protocols-and-stream-oriented-protocols
        // . TLDR: UDP vs TCP. We are using TCP.
        let mut recv_buf = ByteBuf::mut_with_capacity(2048);

        // we are PollOpt::edge() and PollOpt::oneshot(), so we _must_ drain
        // the entire socket receive buffer, otherwise the server will hang.
        loop {
            match self.sock.try_read_buf(&mut recv_buf) {
                // the socket receive buffer is empty, so let's move on
                // try_read_buf internally handles WouldBlock here too
                Ok(None) => {
                    info!("CONN : we read 0 bytes");
                    break;
                },
                Ok(Some(n)) => {
                    info!("CONN : we read {} bytes", n);

                    // if we read less than capacity, then we know the
                    // socket is empty and we should stop reading. if we
                    // read to full capacity, we need to keep reading so we
                    // can drain the socket. if the client sent exactly capacity,
                    // we will match the arm above. the recieve buffer will be
                    // full, so extra bytes are being dropped on the floor. to
                    // properly handle this, i would need to push the data into
                    // a growable Vec<u8>.
                    if n < recv_buf.capacity() {
                        break;
                    }
					
                },
                Err(e) => {
                    println!("Failed to read buffer for token {:?}, error: {}", self.token, e);
                    return Err(e);
                }
            }
        }

        // change our type from MutByteBuf to ByteBuf
        Ok(recv_buf.flip())
    }

    /// Handle a writable event from the event loop.
    ///
    /// Send one message from the send queue to the client. If the queue is empty, remove interest
    /// in write events.
    /// TODO: Figure out if sending more than one message is optimal. Maybe we should be trying to
    /// flush until the kernel sends back EAGAIN?
    pub fn writable(&mut self) -> io::Result<()> {				
		if !self.send_queue.is_empty() && self.send_queue.len() > 0
		{
			info!("Sending message to client");
			try!(self.send_queue.pop()
				.ok_or(Error::new(ErrorKind::Other, "Could not pop send queue"))
				.and_then(|mut buf| {
					match self.sock.try_write_buf(&mut buf) {
						Ok(None) => {
							println!("client flushing buf; WouldBlock");

							// put message back into the queue so we can try again
							self.send_queue.push(buf);
							Ok(())
						},
						Ok(Some(n)) => {
							info!("CONN : we wrote {} bytes", n);
							Ok(())
						},
						Err(e) => {
							println!("Failed to send buffer for {:?}, error: {}", self.token, e);
							Err(e)
						}
					}
				})
			);
		}
		
        if self.send_queue.is_empty() || self.send_queue.len() == 0 {
            self.interest.remove(EventSet::writable());
        }

        Ok(())
    }
	
	/// Welcome socket to the world
	pub fn welcome(&mut self)  -> io::Result<()> {
		match self.sock.try_write(greeting::WELCOME_MESSAGE.as_bytes()) {
			Ok(None) => {
				info!("client flushing buf; WouldBlock");

				// put message back into the queue so we can try again
				self.welcome();
				Ok(())
			},
			Ok(Some(n)) => {
				info!("CONN : we wrote {} bytes", n);
				Ok(())
			},
			Err(e) => {
				println!("Failed to send buffer for {:?}, error: {}", self.token, e);
				Err(e)
			}
		}				
	}

	pub fn send(&mut self, message: String){
		if message.len() > 0
		{
			let now = time::now();	
			let s = format!("[{}:{}:{}]{}", now.tm_hour, now.tm_min, now.tm_sec, message);	
			let bytes = ByteBuf::from_slice(s.as_bytes());	
			self.send_message(bytes);		
		}
	}
	
    /// Queue an outgoing message to the client.
    ///
    /// This will cause the connection to register interests in write events with the event loop.
    /// The connection can still safely have an interest in read events. The read and write buffers
    /// operate independently of each other.
    pub fn send_message(&mut self, message: ByteBuf) -> io::Result<()> {
		info!("send message queued");
        self.send_queue.push(message);
        self.interest.insert(EventSet::writable());
        Ok(())
    }

    /// Register interest in read events with the event_loop.
    ///
    /// This will let our connection accept reads starting next event loop tick.
    pub fn register(&mut self, event_loop: &mut EventLoop<server::Server>) -> io::Result<()> {
        self.interest.insert(EventSet::readable());

        event_loop.register(
            &self.sock,
            self.token,
            self.interest, 
            PollOpt::edge() | PollOpt::oneshot()
        ).or_else(|e| {
            println!("Failed to reregister {:?}, {:?}", self.token, e);
            Err(e)
        })
    }

    /// Re-register interest in read events with the event_loop.
    pub fn reregister(&mut self, event_loop: &mut EventLoop<server::Server>) -> io::Result<()> {
        event_loop.reregister(
            &self.sock,
            self.token,
            self.interest,
            PollOpt::edge() | PollOpt::oneshot()
        ).or_else(|e| {
            println!("Failed to reregister {:?}, {:?}", self.token, e);
            Err(e)
        })
    }
}