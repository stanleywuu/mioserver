pub mod greeting
{
	pub const WELCOME_MESSAGE: &'static str = "Welcome to the mud\r\nWhat's your name?\r\n";
	pub const REGISTER_MESSAGE: &'static str = "This appears to your first time here, \r\nwould you like to visit us in the mud world?\r\n";
	pub const ENTER_PASSWORD: &'static str = "Please enter your pass code\r\n";
	pub const REGISTER_USERNAME: &'static str = "Please enter a username\r\n";
	pub const REGISTER_PASSWORD: &'static str = "Password please:\r\n";
	pub const CONFIRM_PASSWORD: &'static str = "Please confirm your password:\r\n";
	pub const CREATE_CHARACTER: &'static str =	"Let's build your character\r\nWhat would you like to be? \r\n[1]Human\t\t[2]Elf\t\t[3]Dwarf\t\t[4]Dragon\r\n";
}

pub mod character
{
	pub const RACESELECTION: &'static str = "What would you like to be? \r\n[1]Human\t\t[2]Elf\t\t[3]Dwarf\t\t[4]Dragon\r\n";
	pub const GENDERSELECTION: &'static str = "Gender[m/f/u]?\r\n";
	pub const TYPESELECTION: &'static str = "What kind of {} would you like to be?\r\n[1]Intelligent\t\t[2]Atheletic\t\t[3]Average\r\n";
	pub const ATTRSELECTION: &'static str = "Are you satisfied with the following attributes?\r\n";
	pub const SUCCESS: &'static str = "Your character has been created\r\n";
}