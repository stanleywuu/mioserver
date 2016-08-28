use storage;
use Messages::greeting;

#[derive(Copy,Clone,Debug)]
pub enum LogonState
{
	New,
	Username,
	Password,
	RegisterNewUser,
	RegisterPassword,
	RegisterPasswordConfirm,
	RegisterCreation,
	
	
	Done
}

pub struct LogonManager
{
	pub username: String,
	pub password: String,
	pub logon_state: LogonState,
	pub return_msg: String,
}

impl LogonManager
{
	pub fn new_from_data(user:String, pwd:String, state:LogonState, msg:String) -> LogonManager
	{
		LogonManager
		{
			username: user,
			password: pwd,
			logon_state: state,
			return_msg: msg,
		}
	}
	
	pub fn new() -> LogonManager
	{
		LogonManager
		{
			username: String::new(),
			password: String::new(),
			logon_state: LogonState::Username,
			return_msg: String::new(),
		}
	}
}

//check whether user already exists
fn user_exists(username: String) -> bool
{
	let mut db = storage::get_db();
	return db.entry_exists("player", &username[..]);
}

//save user
fn save_player(username: String, password: String)
{
	let mut db = storage::get_db();
	let mut data:Vec<storage::DataColumn> = Vec::new();
	
	data.push(storage::DataColumn::new("password".to_string(), password.clone()));
	data.push(storage::DataColumn::new("stage".to_string(), "creation".to_string()));
	
	match db.insert("player", &username[..], data)
	{
		Ok(_) => println!("Successfully saved user {}|", username),
		Err(e) => println!("Failed to create user {}", e),
	}
}

//process inputs
pub fn process_commands(cmd:String, logon_data: LogonManager) -> LogonManager
{		
	let mut username = logon_data.username;
	let mut password = logon_data.password;
	let mut logon_state = logon_data.logon_state;
	
	let mut input_string = String::new();
	let mut logon_state = logon_state;
	
	println!("State is {:?}", logon_state);
	match logon_state
	{
		LogonState::RegisterCreation => {},
		LogonState::Done => {},
		_=>
		{
			input_string= cmd;
		}
	}
	
	let mut input = input_string.trim().to_string();
	let mut message:String = String::new();
	
	println!("Received command: {} EOF", input);
	match logon_state
	{
		LogonState::New => {
			logon_state = LogonState::Username;
		},
		LogonState::Username =>
		{
			//determines if username exists
			username = input.to_string();
			if !user_exists(username.clone())
			{
				message = greeting::REGISTER_MESSAGE.to_string();
				logon_state = LogonState::RegisterNewUser;
			}
			else
			{
				message = greeting::ENTER_PASSWORD.to_string();
				logon_state = LogonState::Password;
			}
		},
		LogonState::Password =>
		{
			//retrive record and compare password
			logon_state = LogonState::Done;
		}
		LogonState::RegisterNewUser =>
		{
			//checks whether input is y or n
			if input == "y" || input == "yes"
			{
				message = greeting::REGISTER_PASSWORD.to_string();
				logon_state = LogonState::RegisterPassword;
			}
			else
			{
				logon_state = LogonState::New;
				message = greeting::WELCOME_MESSAGE.to_string();	
			}
		},			
		LogonState::RegisterPassword =>
		{
			password = input.to_string();
			
			message = greeting::CONFIRM_PASSWORD.to_string();
			logon_state = LogonState::RegisterPasswordConfirm;				
		},
		LogonState::RegisterPasswordConfirm =>
		{
			if input.to_string() == password
			{
				save_player(username.clone(), password.clone());
				message = greeting::CREATE_CHARACTER.to_string();
				logon_state = LogonState::RegisterCreation;
			}				
			else
			{
				logon_state = LogonState::RegisterPassword;
				message = greeting::REGISTER_PASSWORD.to_string();
			}
		},
		LogonState::RegisterCreation => 
		{
			//character_creator.handle_input();
			logon_state = LogonState::Done;
		},
		
		LogonState::Done => {},
	}

	LogonManager::new_from_data(username, password, logon_state, message)
}