use character;
use character::CharCreator;

use std::vec;
use std::collections::HashMap;

pub struct GameHandler
{
}

impl GameHandler
{	
	pub fn process_commands(cmd:String, data: CharCreator) -> String
	{
		let mut char_info = data.character.info.clone();
		let mut char_attributes = data.character.attr.clone();
		let mut user_name = data.username.clone();
		
		let inputString = cmd.clone();
		inputString
	}	
}