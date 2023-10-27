use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RegisterLocalisedCharacter {
    pub code: String,
    pub name: String,
    pub desc: String,
    pub option1: String,
    pub option1_desc: String,
}

type RegisterLocalisedCharacterList = HashMap<String, RegisterLocalisedCharacter>;

impl RegisterLocalisedCharacter {
    /// `get_character_register_localised` is a function that returns a Result containing either a RegisterLocalisedCharacterList or an error message.
    ///
    /// This function attempts to open a language file for character registration commands from AniList.
    /// The function will return an early error message if it fails to open the file.
    ///
    /// After successfully opening the language file, it reads the content of the file to a string. If it fails while reading the content, it will return an early error message.
    ///
    /// The content of the file is then parsed as JSON and converted into a RegisterLocalisedCharacterList structure. If it fails during the conversion, it will return an error message.
    ///
    /// ## Returns
    /// A `Result<RegisterLocalisedCharacterList, &'static str>` where
    /// * `Ok(RegisterLocalisedCharacterList)` means the function successfully read and parsed the file, returning the data as RegisterLocalisedCharacterList
    /// * `Err(&'static str)` means the function encountered an error at some point. The error message indicates at which point the function failed.
    ///
    /// ## Errors
    /// This function will return `Err("Failed to open file")` if it fails to open the file.
    /// It will return `Err("Failed to read file")` if it fails to read the content of the file.
    /// It will return `Err("Failed to parse json.")` if it cannot parse the file content as JSON.
    ///
    /// # Examples
    /// To use `get_character_register_localised`, you can match on the Result like so:
    /// ```rust
    /// match get_character_register_localised() {
    ///     Ok(data) => println!("Data parsed successfully: {:?}", data),
    ///     Err(e) => println!("Error occurred: {}", e),
    /// }
    /// ```
    pub fn get_character_register_localised() -> Result<RegisterLocalisedCharacterList, &'static str>
    {
        let mut file = match File::open("./lang_file/command_register/anilist/character.json") {
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
