use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RegisterLocalisedUser {
    pub code: String,
    pub name: String,
    pub desc: String,
    pub option1: String,
    pub option1_desc: String,
}

type RegisterLocalisedUserList = HashMap<String, RegisterLocalisedUser>;

impl RegisterLocalisedUser {
    /// Reads and deserializes the JSON data from a specific language file containing user registration information.
    ///
    /// The file path is hard-coded and points to `'./lang_file/command_register/anilist/user.json'`.
    ///
    /// # Errors
    /// This function will return an error if:
    /// * The file doesn't exist or cannot be opened due to a permission error or any other reason (error message: `"Failed to open file"`)
    /// * The file cannot be read (error message: `"Failed to read file"`)
    /// * The JSON content from the file cannot be parsed into the `RegisterLocalisedUserList` struct (error message: `"Failed to parse json."`)
    ///
    /// # Return
    /// The function returns a `Result<RegisterLocalisedUserList, &'static str>`. On success, it provides a `RegisterLocalisedUserList` instance. If any error occurs during file opening, reading, or deserializing, it returns an `Err` variant with a static string describing the error.
    ///
    /// # Example
    ///
    /// ```no_run
    /// match get_user_register_localised() {
    ///     Ok(user_list) => println!("Successfully retrieved user list: {:?}", user_list),
    ///     Err(err) => eprintln!("Error: {}", err),
    /// }
    /// ```
    ///
    pub fn get_user_register_localised() -> Result<RegisterLocalisedUserList, &'static str> {
        let mut file = match File::open("./lang_file/command_register/anilist/user.json") {
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
