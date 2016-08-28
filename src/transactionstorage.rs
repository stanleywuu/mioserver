extern crate rusqlite;
extern crate time;

use std::vec;
use std::path::Path;
use std;
use std::io;
use std::fmt;

pub struct Transaction
{
	pub id: i32,
	pub message: String,
	pub time_created: time::Timespec,
}

impl Transaction
{
	pub fn new(msg:String, created_time: time::Timespec) -> Transaction
	{
		Transaction
		{
			id: 0,
			message: msg,
			time_created: created_time
		}
	}
}

pub struct SqliteDB
{
	pub dbpath: String,
	pub dbconn: rusqlite::Connection
}

impl Clone for SqliteDB
{
	fn clone(&self) -> SqliteDB
	{
		let physical_path = Path::new(self.dbpath.as_str());
		let connection = rusqlite::Connection::open(physical_path).unwrap();
		
		SqliteDB
		{
			dbpath: self.dbpath.clone(),
			dbconn: connection
		}
	}
}


impl SqliteDB
{
	pub fn new(filepath: &str) -> SqliteDB
	{
		let physical_path = Path::new(filepath);
		let connection = rusqlite::Connection::open(physical_path).unwrap();
		
		SqliteDB
		{
			dbpath: String::from(filepath),
			dbconn: connection
		}
	}
	
	pub fn createDB(self)
	{
		// https://github.com/jgallagher/rusqlite
		let result = self.dbconn.execute("CREATE TABLE IF NOT EXISTS messages (
		  id              INTEGER PRIMARY KEY AUTOINCREMENT,
		  message         TEXT NOT NULL,
		  time_created    DATETIME,
		  target		  TEXT
		  )", &[]).unwrap();
	}
	
	pub fn insertRecord(self, record:Transaction)
	{
		let target = String::from("all");
		
		self.dbconn.execute("INSERT INTO messages (message, time_created, target)
		VALUES ($1, $2, $3)", &[&record.message, &record.time_created, &target]).unwrap();
	}
	
	pub fn getRecord(self, updatedTime: time::Timespec) -> Vec<Transaction>
	{
		let mut stmt = self.dbconn.prepare("SELECT id, message, time_created FROM messages 
		WHERE time_created > ?").unwrap();
		let pending_messages = stmt.query_map(&[&updatedTime], |row|
		{
			Transaction
			{
				id: row.get(0),
				message: row.get(1),
				time_created: row.get(2)
			}
		}).unwrap();
		
		let mut messages = Vec::new();
		for message in pending_messages
		{
			messages.push(message.unwrap());
		}
		messages
	}
}



