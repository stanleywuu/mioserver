use std::io::Read;
use std::io::Write;
use std::any::Any;
use std::fs;
use std::fs::File;
use std::io::Error;


pub fn get_db() -> DB
{
	DB
	{
		connection_string: "./db/".to_string(),
	}
}

pub struct DataColumn
{
	column: String,
	data: String,
}

impl DataColumn
{
	pub fn new(col: String, value: String) -> DataColumn
	{
		DataColumn
		{
			column: col,
			data: value
		}
	}
	
	pub fn insert_string(&mut self) -> String
	{
		return format!("{}:{}", self.column, self.data)
	}		
}

pub struct DB
{
	connection_string: String,
}

impl DB
{
	pub fn new(connstring: String) -> DB
	{
		DB
		{
			connection_string: connstring,
		}
	}
	
	pub fn insert(self, table: &str, key:&str, data: Vec<DataColumn>) -> Result<(), Error>
	{
		let dir = self.connection_string.clone() + "//" + table;
		match fs::create_dir_all(dir)
		{
			Ok(ok) => ok,
			Err(_) => {},
		}
		
		let filepath = self.connection_string + "//" + table + "//" + key + ".db";
		
		let mut file = try!(File::create(filepath));
		
		
		let mut content = String::new();
		
		for col in data
		{
			let mut col_str = col;
			content = content + "\n" + &col_str.insert_string();
		}
		file.write_all(content.as_bytes());
		Ok(())
	}
	
	pub fn entry_exists(&mut self, table: &str, key: &str) -> bool
	{
		let filepath = format!("{}//{}//{}.db", self.connection_string, table, key);
		let file = File::open(filepath);
		match file
		{
			Ok(_) =>{println!("file does exist"); return true;},
			Err(_)=>{println!("file does not exist"); return false;},
		}
	}
	
	pub fn read_entry(&self, table: &str, key: &str) -> Result<Vec<DataColumn>, Error>
	{
		let filepath = format!("{}//{}//{}.db", self.connection_string, table, key);
		let file = File::open(filepath);
		let mut content = String::new();
		let mut data: Vec<DataColumn> = Vec::new();
		
		match file
		{
			Ok(f) => 
			{
				let mut file_ps = f;
				try!(file_ps.read_to_string(&mut content));
				let lines = content.lines();
				
				for line in lines
				{
					let parts: Vec<&str> = line.split(':').collect();
					data.push(DataColumn::new
						(parts[0].to_string(), 
						if parts.len() > 1 {parts[1].to_string()}
						else {String::new()}
						));
				}
			},
			Err(_) => {},
		}
		Ok(data)
	}
}