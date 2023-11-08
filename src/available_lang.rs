use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AvailableLang {
    pub lang: String,
}

type AvailableLangList = HashMap<String, AvailableLang>;

impl AvailableLang {
    /// Reads and returns a list of available languages from a JSON file.
    ///
    /// This function opens a file named `available_lang.json` in the `lang_file` directory.
    /// After opening the file, it reads the JSON content into a `String`.
    /// Finally, it tries to deserialize the JSON content into an `AvailableLangList` object.
    ///
    /// # Errors
    ///
    /// If any error occurs while performing these operations, such as:
    ///
    /// * Error while opening the file.
    /// * Error while reading the file.
    /// * Error while parsing the JSON content.
    ///
    /// This function will return a static string describing the error.
    ///
    /// # Returns
    ///
    /// A `Result` which is:
    ///
    /// * `Ok(AvailableLangList)` if all operations completed successfully.
    /// * `Err(&'static str)` if any error occurred.
    ///
    /// # Example
    ///
    /// ```
    /// use crate::your_module::get_available_lang;
    ///
    /// match get_available_lang() {
    ///     Ok(lang_list) => println!("Available languages: {:?}", lang_list),
    ///     Err(err) => println!("Error: {}", err),
    /// }
    /// ```
    pub fn get_available_lang() -> Result<AvailableLangList, &'static str> {
        let mut file = match File::open("./lang_file/available_lang.json") {
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
