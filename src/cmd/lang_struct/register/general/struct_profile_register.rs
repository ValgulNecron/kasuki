use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RegisterLocalisedProfile {
    pub code: String,
    pub name: String,
    pub desc: String,
    pub option1: String,
    pub option1_desc: String,
}

type RegisterLocalisedProfileList = HashMap<String, RegisterLocalisedProfile>;

impl RegisterLocalisedProfile {
    /// # get_profile_register_localised
    ///
    /// Opens a file named "lang_file/command_register/general/profile.json", reads its contents and deserialises it into a `RegisterLocalisedProfileList`. Returns a `Result` indicating the success or failure of these operations.
    ///
    /// # Returns
    ///
    /// If successful, this function returns a `Result::Ok` containing the deserialised `RegisterLocalisedProfileList` from the JSON file.
    ///
    /// If the function fails at any point, it returns a `Result::Err` with an appropriate message indicating what part of the operation failed:
    /// - "Failed to open file"
    /// - "Failed to read file"
    /// - "Failed to parse json."
    ///
    /// # Errors
    ///
    /// This function returns `&'static str` error in form of a `Result::Err` when:
    /// - The JSON file could not be opened (for any reason)
    /// - The file contents could not be read (for any reason)
    /// - The JSON data could not be deserialised into a `RegisterLocalisedProfileList` (for any reason)
    ///
    /// # Example
    ///
    /// ```rust
    /// let profile = get_profile_register_localised();
    /// match profile {
    ///     Ok(data) => println!("Data: {:?}", data),
    ///     Err(err) => println!("Error: {}", err),
    /// }
    /// ```
    ///
    pub fn get_profile_register_localised() -> Result<RegisterLocalisedProfileList, &'static str> {
        let mut file = match File::open("lang_file/command_register/general/profile.json") {
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
