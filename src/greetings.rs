use connection;
use bytes::{Buf, RingBuf, SliceBuf, MutBuf, ByteBuf};

enum ConnectionState
{
	New,
	Logon,
	Play,
}

pub struct ConnManager<'a>
{
	pub conn: &'a connection::Connection,
	state: ConnectionState,
}

impl<'a> ConnManager<'a>
{
	pub fn new(conn: &'a connection::Connection) -> ConnManager<'a>
	{
		ConnManager
		{
			conn: conn,
			state: ConnectionState::New,
		}
	}
	
	pub fn handle_input(&self, input: ByteBuf)
	{
		match self.state
			{
				ConnectionState::New =>
				{	
					info!("greeting message");
					//let mut logon_mgr = logon::logon_manager::new(conn);
					//logon_mgr.handle_logon();					
				},
				_ => {}
			}
	}
}