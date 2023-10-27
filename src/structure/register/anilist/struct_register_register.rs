use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RegisterLocalisedRegister {
    pub code: String,
    pub name: String,
    pub desc: String,
    pub option1: String,
    pub option1_desc: String,
}

type RegisterLocalisedRegisterList = HashMap<String, RegisterLocalisedRegister>;

impl RegisterLocalisedRegister {
    /// Function: `get_register_register_localised`
    ///
    /// This is a public function that reads a JSON file from the disk, parses it into a
    /// `RegisterLocalisedRegisterList` and returns the result or an error string if any of
    /// the operations fails.
    ///
    /// # Return
    ///
    /// The function returns a `Result` holding either a:
    /// * `RegisterLocalisedRegisterList` - parsed successfully from the JSON file.
    /// * `&'static str` - static string representing an error message.
    ///
    /// # Errors
    ///
    /// The function can fail on the following cases:
    /// * If the file is not found, cannot be accessed or opened for some reason,
    ///   it will return the string "Failed to open file".
    /// * If the file cannot be read into a string, it will return "Failed to read file".
    /// * If the JSON string cannot be parsed into a `RegisterLocalisedRegisterList`,
    ///   it will return "Failed to parse json."
    ///
    /// # Example
    ///
    /// ```
    /// use your_module_name::get_register_register_localised;
    ///
    /// let result = get_register_register_localised();
    ///
    /// match result {
    ///     Ok(data) => println!("Data: {:?}", data),
    ///     Err(e) => println!("Error happened: {}", e),
    /// }
    /// ```
    ///
    /// # Panics
    ///
    /// This function doesn't panic.
    pub fn get_register_register_localised() -> Result<RegisterLocalisedRegisterList, &'static str>
    {
        let mut file = match File::open("./lang_file/command_register/anilist/register.json") {
            Ok(file) => file,
            Err(_) => return Err("Failed to open file"),
        };
        let mut json = String::new();
        match file.read_to_string(&mut json) {
            Ok(_) => {}
            Err(_) => return Err("Failed to read file"),
        };
        match serde_json::from_str(&json) {
            Ok(data) => Ok(data),
            Err(_) => Err("Failed to parse json."),
        }
    }
}
