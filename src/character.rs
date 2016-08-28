extern crate rand;

use self::rand::Rng;

use std::vec;
use std::collections::HashMap;
use storage;
use Messages::character;

pub mod debug
{
	use std::collections::HashMap;
	
	pub fn print_hashmap(hash:HashMap<String,String>)
	{
		info!("----Printing hash:---");
		for (key, value) in hash
		{
			info!("[{}]:{}---", key, value);
		}
	}
}

fn save_character(character: Character)
{
	let mut db = storage::get_db();
	let mut data:Vec<storage::DataColumn> = Vec::new();
	
	let info = character.info.clone();
	let attr = character.attr.clone();
	
	for (key, value) in info
	{
		data.push(storage::DataColumn::new(key, value));
	}
	
	for (key, value) in attr
	{
		data.push(storage::DataColumn::new(key, value.to_string()));
	}
	
	match db.insert("player_char_info", &character.username[..], data)
	{
		Ok(_) => println!("Successfully saved character {}|", character.username),
		Err(e) => println!("Failed to save character {}", e),
	}
}
	
pub struct Character
{
	pub username: String,
	pub info: HashMap<String, String>,
	pub attr: HashMap<String, i32>,
	pub items: Vec<()>,
	pub equips: Vec<()>,
	pub skills: Vec<()>,
	pub history: Vec<()>,
}

impl Character
{
	pub fn new() -> Character
	{
		Character
		{
			username: String::new(),
			info: HashMap::new(),
			attr: HashMap::new(),
			items: Vec::new(),
			equips: Vec::new(),
			skills: Vec::new(),
			history: Vec::new(),
		}
	}
	
	pub fn new_from_data(id: String, char_info: HashMap<String, String>, char_attr: HashMap<String, i32>) -> Character
	{
		Character
		{
			username: id,
			info: char_info,
			attr: char_attr,
			items: Vec::new(),
			equips: Vec::new(),
			skills: Vec::new(),
			history: Vec::new(),
		}
	}
}

pub fn initialize_info() -> HashMap<String, String>
{
	let mut info = HashMap::new();
	
	info.insert("name".to_string(), String::new());
	info.insert("race".to_string(), String::new());
	info.insert("personality".to_string(), String::new());
	info.insert("description".to_string(), String::new());
	info.insert("look".to_string(), String::new());
	
	info
}

pub fn initialize_attr() -> HashMap<String, i32>
{
	let mut attr = HashMap::new();
	
	attr.insert("hp".to_string(), 10);
	attr.insert("stam".to_string(), 10);
	attr.insert("mana".to_string(), 10);
	attr.insert("agi".to_string(), 3);
	attr.insert("str".to_string(), 3);
	attr.insert("magic".to_string(), 3);
	
	attr

}

pub fn initialize_attr_with_bias(max_points: i32, weights: HashMap<String, i32>) -> HashMap<String, i32>
{
	let mut attrs = HashMap::new();
	//average value
	let mut size = weights.len() as i32;
	let mut average = max_points / size;
	let mut points_left = max_points;
	let mut calculated = 0;
	
	for (attr, weight) in weights
	{
		//To calculate weight, find the difference between average and weight
		average = points_left / (size - calculated);
		let difference = weight - average;
		let val = rand::thread_rng().gen_range(2, average) + difference;
		
		update_attr(&mut attrs, attr, weight);
		
		calculated += 1;
		points_left = points_left - val;				
	}
	
	attrs
}

fn update_attr(info: &mut HashMap<String, i32>, key: String, value: i32)
{
	if !info.contains_key(&key)
	{
		info.insert(key, value);
	}
	else if let Some(x) = info.get_mut(&key)
	{
		*x = value;
	}
}


fn update_info(info: &mut HashMap<String, String>, key: String, value: String)
{
	if !info.contains_key(&key)
	{
		info.insert(key, value.trim().to_string());
	}
	else if let Some(x) = info.get_mut(&key)
	{
		*x = value.trim().to_string();
	}
}

fn get_weightings(race_selected: i32) -> HashMap<String, i32>
{
	let mut weighting = HashMap::new();
	let mut a_agi = 3;
	let mut a_str = 3;
	let mut a_int = 3;
	let mut a_charm = 3;
		
	if race_selected == 1
	{
		weighting.insert("str".to_string(), 5);
		weighting.insert("agi".to_string(), 2);
		weighting.insert("int".to_string(), 3);
		weighting.insert("charm".to_string(),3);
	}
	if race_selected == 2
	{
		weighting.insert("str".to_string(), 3);
		weighting.insert("agi".to_string(), 5);
		weighting.insert("int".to_string(), 3);
		weighting.insert("charm".to_string(),2);
	}
	if race_selected == 3
	{
		weighting.insert("str".to_string(), 2);
		weighting.insert("agi".to_string(), 2);
		weighting.insert("int".to_string(), 4);
		weighting.insert("charm".to_string(),5);
	}
	if race_selected == 4
	{
		weighting.insert("str".to_string(), 3);
		weighting.insert("agi".to_string(), 3);
		weighting.insert("int".to_string(), 3);
		weighting.insert("charm".to_string(),3);
	}
	
	weighting
}

pub fn update_multi(info: &mut HashMap<String, String>, data: HashMap<String, String>)
{
	for (key, value) in data
	{
		if !info.contains_key(&key)
		{
			info.insert(key,value);
		}
		else if let Some(x) = info.get_mut(&key)
		{
			*x = value;
		}
	}
}

#[derive(Copy,Clone,Debug)]
pub enum CreationState
{
	New,
	Race,
	Gender,
	Type,
	Selection,
	
	Done
}

pub struct CharCreator
{
	pub username: String,
	pub character: Character,
	pub state: CreationState,
	pub return_msg: String,
}

impl CharCreator
{
	pub fn new() -> CharCreator
	{
		CharCreator
		{
			username: String::new(),
			character: Character::new(),
			state: CreationState::Race,
			return_msg: String::new(),
		}
	}
	pub fn new_from_data(user_name: String, char: Character, state: CreationState, msg: String) -> CharCreator
	{
		CharCreator
		{
			username: user_name,
			character: char,
			state: state,
			return_msg: msg,
		}
	}
	
	fn is_input_valid(min:i32, max:i32, input: String) -> i32
	{
		let selection: i32 = match input.trim().parse()
		{	Ok(num) => num,
			Err(_) => -1,
		};
		
		
		if selection >= min && selection <= max
		{
			selection
		}
		else
		{
			0
		}
	}
	
	fn is_string_valid(selection: String, input:String) -> bool
	{
		let parts: Vec<&str> = selection.split(':').collect();
		let mut valid = false;
		
		for part in parts
		{
			if input.trim() == part {valid = true; break;}				
		}
		
		valid
	}
	
	pub fn process_commands(cmd:String, data: CharCreator) -> CharCreator
	{
		let state = data.state;	
		let mut creation_state: CreationState = CreationState::New;
		let mut message = String::new();
		let mut char_info = data.character.info.clone();
		let mut char_attributes = data.character.attr.clone();
		let mut user_name = data.username.clone();
		
		debug::print_hashmap(char_info.clone());
				
		match state
		{
			CreationState::New=>
			{
				message = character::RACESELECTION.to_string();
				creation_state = CreationState::Race;
			}
			CreationState::Race=>{
				let selection = CharCreator::is_input_valid(1, 4, cmd);
				char_info = initialize_info();
				char_attributes = initialize_attr();
				
				//save race
				if selection > 0
				{
					update_info(&mut char_info, "race".to_string(), selection.to_string());
						
					message = character::GENDERSELECTION.to_string();
					creation_state = CreationState::Gender;
				}
				else
				{
					creation_state = CreationState::New;
				}
			},
			CreationState::Gender=>
			{
				//save gender
				if CharCreator::is_string_valid("m:f:u".to_string(), cmd.clone())
				{
					message = character::TYPESELECTION.to_string();
					
					update_info(&mut char_info, "gender".to_string(), cmd.clone());
					debug::print_hashmap(char_info.clone());
					creation_state = CreationState::Type;
				}
				else
				{
					message = character::GENDERSELECTION.to_string();
				}
			},
			CreationState::Type => 
			{
				let selection = CharCreator::is_input_valid(0, 4, cmd);
				if  selection > 0
				{
					message = character::ATTRSELECTION.to_string() + "\n";
					update_info(&mut char_info, "type".to_string(), selection.to_string().clone());					
					
					char_attributes = initialize_attr_with_bias(15,get_weightings(selection));
					
					for (key, value) in char_attributes.clone()
					{
						message = message + &key + ":" + &value.to_string();
					}
					
					creation_state = CreationState::Selection;
				}
				else
				{
					message = character::TYPESELECTION.to_string();
				}
			},
			CreationState::Selection =>
			{
				let input = cmd.clone();
				if input.trim() == "y" || input.trim() == "yes"
				{
					message = character::SUCCESS.to_string();
					creation_state = CreationState::Done;
					//save
					let char = Character::new_from_data(user_name.clone(), char_info.clone(), char_attributes.clone());
					save_character(char);
				}
				else
				{
					message = character::TYPESELECTION.to_string();
					creation_state = CreationState::Type;
				}
			},
			CreationState::Done =>
			{
			}
			
		}
		
		let character = Character::new_from_data(user_name.clone(), char_info.clone(), char_attributes.clone());
		CharCreator::new_from_data(user_name.clone(), character, creation_state, message)
	}
	
}