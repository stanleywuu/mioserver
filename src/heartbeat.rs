extern crate time;

use connection;

use transactionstorage;
use transactionstorage::SqliteDB;
use transactionstorage::Transaction;

use std;
use std::vec;

pub struct HeartbeatHandler
{
	Client: connection::Connection,
	DBConnection: SqliteDB
}

impl HeartbeatHandler
{
	pub fn new(client: connection::Connection) -> HeartbeatHandler
	{
		HeartbeatHandler
		{
			Client: client,
			DBConnection: db.clone()
		}
	}
	
	pub fn process_messages(self)
	{
		
	}
}