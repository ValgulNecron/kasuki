use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RegisterLocalisedStudio {
    pub code: String,
    pub name: String,
    pub desc: String,
    pub option1: String,
    pub option1_desc: String,
}

type RegisterLocalisedStudioList = HashMap<String, RegisterLocalisedStudio>;

impl RegisterLocalisedStudio {
    /// # get_studio_register_localised
    ///
    /// This function fetches the localised studio information from a JSON file.
    ///
    /// ## Returns
    ///
    /// This function returns a `Result<RegisterLocalisedStudioList, &'static str>`. If the function is successful in reading
    /// and parsing the JSON file, it will return `Ok(RegisterLocalisedStudioList)`. In case of any error while opening, reading
    /// or parsing the JSON file, it will return an `Err(&'static str)`. The error message describes what caused the failure.
    ///
    /// ## Errors
    ///
    /// This function will return an error in the following circumstances:
    ///
    /// - If the JSON file fails to open, the error message will be "Failed to open file".
    ///
    /// - If the JSON file fails to be read, the error message will be "Failed to read file".
    ///
    /// - If the JSON string from the file can not be parsed, the error message will be "Failed to parse json".
    ///
    /// ## Examples
    ///
    /// ```Rust
    /// extern crate your_crate;
    ///
    /// use your_crate::get_studio_register_localised;
    ///
    /// let result = get_studio_register_localised();
    ///
    /// match result {
    ///     Ok(data) => println!("Successfully fetched data."),
    ///     Err(e) => {
    ///         println!("Failed with error: {}: {}", e);
    ///     }
    /// }
    /// ```
    pub fn get_studio_register_localised() -> Result<RegisterLocalisedStudioList, &'static str> {
        let mut file = match File::open("./lang_file/command_register/anilist/studio.json") {
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
