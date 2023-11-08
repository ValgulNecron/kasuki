use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RegisterLocalisedLN {
    pub code: String,
    pub name: String,
    pub desc: String,
    pub option1: String,
    pub option1_desc: String,
}

type RegisterLocalisedLNList = HashMap<String, RegisterLocalisedLN>;

impl RegisterLocalisedLN {
    /// This function handles the process of localizing the register by fetching
    /// the localised version from a local json file.
    ///
    /// # Errors
    ///
    /// The function can encounter the following types of errors:
    /// 1. It fails to open the local json file (`./lang_file/command_register/anilist/ln.json`).
    /// 2. It fails to read the file contents into a string.
    /// 3. It fails to parse the string content into a `RegisterLocalisedLNList` object.
    /// In each of these cases, the function will terminate and return an appropriate error message as a string.
    ///
    /// # Returns
    ///
    /// The function returns a `Result` object. This object contains a `RegisterLocalisedLNList` struct
    /// when the operation is successful or an error message when the operation fails.
    ///
    /// The function is designed to be used in the context of a registration process.
    /// It supports localization, thereby providing flexibility and a better user experience.
    ///
    /// It is a public interface, meaning it can be accessed from modules outside the one it is defined in.
    ///
    /// Note: You need to make sure `"./lang_file/command_register/anilist/ln.json"` file should be available
    /// and in correct format.
    ///
    /// # Example usage:
    ///
    /// ```
    /// let result = get_ln_register_localised();
    ///
    /// match result {
    ///     Ok(localised_data) => {
    ///         // use localised_data
    ///     }
    ///     Err(err_msg) => {
    ///         // handle error
    ///     }
    /// }
    /// ```
    pub fn get_ln_register_localised() -> Result<RegisterLocalisedLNList, &'static str> {
        let mut file = match File::open("./lang_file/command_register/anilist/ln.json") {
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
