use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RegisterLocalisedAddActivity {
    pub code: String,
    pub name: String,
    pub desc: String,
    pub option1: String,
    pub option1_desc: String,
    pub option2: String,
    pub option2_desc: String,
}

type RegisterLocalisedAddActivityList = HashMap<String, RegisterLocalisedAddActivity>;

impl RegisterLocalisedAddActivity {
    /// This function is used to read a local language JSON file (`add_activity.json`) and convert it to
    /// `RegisterLocalisedAddActivityList` type. It is located in the path `./lang_file/command_register/anilist/anime_activity/add_activity.json`.
    ///
    /// The purpose of this function is to facilitate internationalization/localization by providing
    /// a way to read localized strings from a configuration file.
    ///
    /// # Returns
    ///
    /// On Success: A `Result` wrapping `RegisterLocalisedAddActivityList`. This contains a list of localized
    /// strings for adding activities.
    ///
    /// On Failure: A `Result` wrapping a static string, indicating the error reason.
    ///
    /// - Fails when the file could not be opened ("Failed to open file").
    /// - Fails when the file could not be read into a string ("Failed to read file").
    /// - Fails when the string could not be parsed into json ("Failed to parse json").
    ///
    /// # Example
    ///
    /// ```Rust
    /// let result = get_add_activity_register_localised();
    /// match result {
    ///     Ok(data) => println!("{:?}", data),
    ///     Err(e) => println!("Error: {}", e),
    /// }
    /// ```
    ///
    /// # Note
    ///
    /// The error messages are static and not particularly descriptive. As an enhancement, you might want to
    /// return more descriptive error types, such as using `std::io::Error` or your own custom error type.
    pub fn get_add_activity_register_localised(
    ) -> Result<RegisterLocalisedAddActivityList, &'static str> {
        let mut file = match File::open(
            "./lang_file/command_register/anilist/anime_activity/add_activity.json",
        ) {
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
