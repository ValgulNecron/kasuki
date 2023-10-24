use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RegisterLocalisedCredit {
    pub code: String,
    pub name: String,
    pub desc: String,
}

type RegisterLocalisedCreditList = HashMap<String, RegisterLocalisedCredit>;

impl RegisterLocalisedCredit {
    /// Opens a file containing credit register information,
    /// reads the content, and parses it from JSON into a `RegisterLocalisedCreditList` data structure.
    ///
    /// # Errors
    ///
    /// This function will return an error if it encounters these situations:
    /// * it fails to open the specified file
    /// * it fails to read the file content
    /// * it fails to parse the JSON content in the file
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use your_module_name::get_credit_register_localised;
    ///
    /// match get_credit_register_localised() {
    ///     Ok(data) => println!("{:?}", data),
    ///     Err(e) => println!("An error occurred: {}", e),
    /// }
    /// ```
    ///
    /// # Returns
    ///
    /// A `Result` with `RegisterLocalisedCreditList` when successful, or a static string describing the error otherwise.
    pub fn get_credit_register_localised() -> Result<RegisterLocalisedCreditList, &'static str> {
        let mut file = match File::open("lang_file/command_register/general/credit.json") {
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
