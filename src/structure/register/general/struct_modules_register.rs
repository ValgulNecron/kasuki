use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RegisterLocalisedModule {
    pub code: String,
    pub name: String,
    pub desc: String,
    pub option1: String,
    pub option1_desc: String,
    pub option2: String,
    pub option2_desc: String,
}

type RegisterLocalisedModuleList = HashMap<String, RegisterLocalisedModule>;

impl RegisterLocalisedModule {
    /// `get_module_register_localised` is a function that reads a localisation file
    /// and parses it into a `RegisterLocalisedModuleList`.
    ///
    /// The function attempts to open a JSON file, read the file's contents into a `String`,
    /// and then parse that `String` into a `RegisterLocalisedModuleList` using the `serde_json::from_str` function.
    ///
    /// # Returns
    ///
    /// Returns a `Result`. If successful, the function returns `Ok(RegisterLocalisedModuleList)`.
    /// If it fails at any point (opening, reading, or parsing the file), it will return `Err(&'static str)` containing a relevant error message.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    ///
    /// * The file fails to open (returns `Err("Failed to open file")`)
    /// * The file fails to read (returns `Err("Failed to read file")`)
    /// * The JSON fails to parse (returns `Err("Failed to parse json.")`)
    ///
    /// # Example
    ///
    /// ```
    /// let result = get_module_register_localised();
    /// match result {
    ///     Ok(data) => println!("Data: {:?}", data),
    ///     Err(e) => println!("Error: {:?}", e),
    /// }
    /// ```
    pub fn get_module_register_localised() -> Result<RegisterLocalisedModuleList, &'static str> {
        let mut file =
            match File::open("./lang_file/command_register/general/module_activation.json") {
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
