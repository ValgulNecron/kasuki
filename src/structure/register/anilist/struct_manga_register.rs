use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RegisterLocalisedManga {
    pub code: String,
    pub name: String,
    pub desc: String,
    pub option1: String,
    pub option1_desc: String,
}

type RegisterLocalisedMangaList = HashMap<String, RegisterLocalisedManga>;

impl RegisterLocalisedManga {
    /// `get_manga_register_localised` is a function in the Rust programming language.
    ///
    /// This function is used to read and parse a JSON file that contains localised manga registration information.
    ///
    /// # Returns
    ///
    /// This function returns a `Result` variant. The `Ok` variant contains a `RegisterLocalisedMangaList` representing the parsed information from the JSON file. If there is an error when trying to open, read, or parse the file, the function returns an `Err` variant containing a static string detailing the error that occurred.
    ///
    /// # Errors
    ///
    /// This function will return an error in the following situations:
    ///
    /// * The file cannot be opened (returns "Failed to open file").
    /// * The file cannot be read (returns "Failed to read file").
    /// * The JSON in the file cannot be parsed (returns "Failed to parse json.").
    ///
    /// # Example
    ///
    /// ```Rust
    /// let result = get_manga_register_localised();
    /// match result {
    ///     Ok(register) => /* process the register */,
    ///     Err(e) => println!("An error occurred: {}", e),
    /// }
    /// ```
    ///
    pub fn get_manga_register_localised() -> Result<RegisterLocalisedMangaList, &'static str> {
        let mut file = match File::open("./lang_file/command_register/anilist/manga.json") {
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
