use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RegisterLocalisedPing {
    pub code: String,
    pub name: String,
    pub desc: String,
}

type RegisterLocalisedPingList = HashMap<String, RegisterLocalisedPing>;

impl RegisterLocalisedPing {
    /// `get_ping_register_localised` is a function that attempts to open and parse a JSON file located at "lang_file/command_register/general/ping.json".
    ///
    /// # Returns
    ///
    /// This function returns a `Result` with either a successfully parsed data in `RegisterLocalisedPingList` or an error message of type `&'static str` that indicating the kind of error that occurred.
    /// Available error messages are:
    /// * "Failed to open file": It could not locate or access the file at the specified path.
    /// * "Failed to read file": It could open the file, but could not read it.
    /// * "Failed to parse json.": It could read the file, but could not parse the JSON.
    ///
    /// # Errors
    ///
    /// This function will return an error if it can't open the JSON file, read its contents or parse the JSON string.
    ///
    /// # Examples
    ///
    /// Below is a hypothetical usage example of this function:
    ///
    /// ```rust
    /// let ping_list = get_ping_register_localised();
    /// match ping_list {
    ///     Ok(list) => {
    ///         println!("{:?}", list);
    ///     },
    ///     Err(message) => {
    ///        println!("Error: {}", message);
    ///     }
    /// }
    /// ```
    pub fn get_ping_register_localised() -> Result<RegisterLocalisedPingList, &'static str> {
        let mut file = match File::open("lang_file/command_register/general/ping.json") {
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
