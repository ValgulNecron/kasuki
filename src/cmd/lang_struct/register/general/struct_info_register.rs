use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RegisterLocalisedInfo {
    pub code: String,
    pub name: String,
    pub desc: String,
}

type RegisterLocalisedInfoList = HashMap<String, RegisterLocalisedInfo>;

impl RegisterLocalisedInfo {
    /// Retrieves a list of localised languages for profile registration from a file.
    ///
    /// This function opens a "lang_file/command_register/general/lang.json" file and reads it into a string.
    /// It then parses the JSON string into a series of `RegisterLocalisedLangList` representations.
    ///
    /// # Errors
    ///
    /// The function will return an `Err` variant of `Result` when:
    /// - It fails to open the file (e.g., the file does not exist, or the application does not have sufficient permissions to open it).
    /// - It fails to read the file content.
    /// - It fails to parse the JSON content into `RegisterLocalisedLangList`.
    ///
    /// Each error variant returns a static string detailing the cause of the failure.
    ///
    /// # Returns
    ///
    /// On success, the function returns a `RegisterLocalisedLangList` object parsed from JSON content.
    ///
    /// # Example
    ///
    /// ```
    /// let result = get_profile_register_localised();
    /// match result {
    ///     Ok(data) => println!("{:?}", data),
    ///     Err(e) => eprintln!("An error occurred: {}", e),
    /// }
    /// ```
    ///
    /// # Note
    ///
    /// Depending on the context where you use this function, error handling mechanisms other than matching result and printing error may be more appropriate.
    pub fn get_info_register_localised() -> Result<RegisterLocalisedInfoList, &'static str> {
        let mut file = match File::open("lang_file/command_register/general/info.json") {
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
