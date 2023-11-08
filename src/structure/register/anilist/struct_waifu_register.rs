use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RegisterLocalisedWaifu {
    pub code: String,
    pub name: String,
    pub desc: String,
    pub option1: String,
    pub option1_desc: String,
}

type RegisterLocalisedWaifuList = HashMap<String, RegisterLocalisedWaifu>;

impl RegisterLocalisedWaifu {
    /// Function name:  get_waifu_register_localised
    ///
    /// # Description
    /// This function is responsible for reading the localised Waifu list from the JSON file located at './lang_file/command_register/anilist/waifu.json'.
    /// The functions attempts to open the specified file, read its content into a string then attempts to parse the JSON content of string into 'RegisterLocalisedWaifuList' type data.
    ///
    /// # Errors
    /// This function will return an error if the file can't be opened, can't be read, or if the JSON parsing fails.
    ///
    /// # Returns
    /// It returns a Result. If successful, it returns a type of 'RegisterLocalisedWaifuList'. If unsuccessful, it returns a static string indicating the type of error.
    ///
    /// # Examples
    /// ```
    /// # use your_package_name::get_waifu_register_localised;
    ///
    /// let result = get_waifu_register_localised();
    ///
    /// match result {
    ///     Ok(waifu_list) => println!("Waifu list retrieved successfully"),
    ///     Err(err) => println!("There was an error: {}", err),
    /// }
    /// ```
    pub fn get_waifu_register_localised() -> Result<RegisterLocalisedWaifuList, &'static str> {
        let mut file = match File::open("./lang_file/command_register/anilist/waifu.json") {
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
