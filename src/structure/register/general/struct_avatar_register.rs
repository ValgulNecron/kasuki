use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RegisterLocalisedAvatar {
    pub code: String,
    pub name: String,
    pub description: String,
    pub option1: String,
    pub option1_desc: String,
}

type RegisterLocalisedAvatarList = HashMap<String, RegisterLocalisedAvatar>;

impl RegisterLocalisedAvatar {
    /// Function `get_avatar_register_localised` attempts to read a json configuration file and parse it into a RegisterLocalisedAvatarList.
    ///
    /// The function tries to open a file "lang_file/command_register/general/avatar.json", read it into a string and then, by using `serde_json` crate, parse the JSON string into RegisterLocalisedAvatarList.
    ///
    /// # Errors
    ///
    /// The function can return an error in three possible cases:
    /// - If the file cannot be opened. In this case, it returns a static string error "Failed to open file".
    /// - If the file cannot be read to a string. Then it returns "Failed to read file".
    /// - If the parsed string cannot be converted to RegisterLocalisedAvatarList. In this case it returns "Failed to parse json."
    ///
    /// # Returns
    ///
    /// A `Result` type is returned. If successful, it will contain `Ok(RegisterLocalisedAvatarList)` where RegisterLocalisedAvatarList is the parsed JSON data;
    /// If it fails at any point, it would contain `Err(&'static str)` that tells the user about the specific error.
    ///
    /// # Example
    ///
    /// ```
    /// fn main() {
    ///     match get_avatar_register_localised() {
    ///         Ok(data) => println!("Data: {:?}", data),
    ///         Err(e) => println!("An error occurred: {}", e),
    ///     }
    /// }
    /// ```
    ///
    pub fn get_avatar_register_localised() -> Result<RegisterLocalisedAvatarList, &'static str> {
        let mut file = match File::open("./lang_file/command_register/general/avatar.json") {
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
