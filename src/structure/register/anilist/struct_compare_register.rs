use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RegisterLocalisedCompare {
    pub code: String,
    pub name: String,
    pub desc: String,
    pub option1: String,
    pub option1_desc: String,
    pub option2: String,
    pub option2_desc: String,
}

type RegisterLocalisedCompareList = HashMap<String, RegisterLocalisedCompare>;

impl RegisterLocalisedCompare {
    /// # get_compare_register_localised
    ///
    /// This function attempts to open a JSON file located at `./lang_file/command_register/anilist/compare.json`.
    /// After successfully opening the file, it reads the content into a string, then parses the JSON content
    /// into a `Result` of a `RegisterLocalisedCompareList` object or a static string error message.
    ///
    /// ## Return
    ///
    /// If successful, the function returns `Result::Ok(RegisterLocalisedCompareList)` where `RegisterLocalisedCompareList`
    /// is the object representation of the parsed JSON content.
    /// If an error occurs during any operation (open, read, or parse), the function returns `Result::Err(&'static str)`,
    /// with the error message explaining the failed operation.
    ///
    /// ## Errors
    ///
    /// The function can return three possible error messages:
    /// * "Failed to open file" - If the system cannot open the JSON file.
    /// * "Failed to read file" - If the system cannot read the content of the file.
    /// * "Failed to parse json." - If the content of the file could not be correctly parsed to a `RegisterLocalisedCompareList` object.
    ///
    /// ## Example
    ///
    /// ```rust
    /// let result = get_compare_register_localised();
    /// match result {
    ///     Ok(data) => println!("{:?}", data),
    ///     Err(e) => println!("Error: {}", e),
    /// }
    /// ```
    pub fn get_compare_register_localised() -> Result<RegisterLocalisedCompareList, &'static str> {
        let mut file = match File::open("./lang_file/command_register/anilist/compare.json") {
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
