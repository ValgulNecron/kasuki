use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RegisterLocalisedStaff {
    pub code: String,
    pub name: String,
    pub desc: String,
    pub option1: String,
    pub option1_desc: String,
}

type RegisterLocalisedStaffList = HashMap<String, RegisterLocalisedStaff>;

impl RegisterLocalisedStaff {
    /// Opens and reads from a localized staff registration file in JSON format.
    ///
    /// This function performs the following steps:
    /// 1. Open the file "./lang_file/command_register/anilist/staff.json".
    /// 2. Read the contents of the file into a string.
    /// 3. Parse the string as JSON using Serde.
    ///
    /// # Returns
    ///
    /// * On success, a `std::result::Result::Ok` value is returned containing an instance of `RegisterLocalisedStaffList`,
    /// constructed from the parsed JSON.
    /// * If the file cannot be opened, read, or the JSON cannot be parsed, a `std::result::Result::Err` value is returned with an error message.
    ///
    /// # Errors
    ///
    /// This function will return an error if the file cannot be opened, the file cannot be read into a string, or the string cannot be parsed as JSON.
    ///
    /// # Example
    ///
    /// ```Rust
    /// let result = get_staff_register_localised();
    /// match result {
    ///     Ok(data) => println!("Staff register: {:?}", data),
    ///     Err(e) => println!("An error occurred: {}", e),
    /// }
    /// ```
    pub fn get_staff_register_localised() -> Result<RegisterLocalisedStaffList, &'static str> {
        let mut file = match File::open("./lang_file/command_register/anilist/staff.json") {
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
