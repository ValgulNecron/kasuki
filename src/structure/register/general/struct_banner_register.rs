use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RegisterLocalisedBanner {
    pub code: String,
    pub name: String,
    pub description: String,
    pub option1: String,
    pub option1_desc: String,
}

type RegisterLocalisedBannerList = HashMap<String, RegisterLocalisedBanner>;

impl RegisterLocalisedBanner {
    /// This function reads and parses a JSON file located at "lang_file/command_register/general/banner.json".
    ///
    /// It uses `File::open()` to open the file and `read_to_string()` to read the contents of the file into a `String`.
    /// It uses `serde_json::from_str()` to parse it into a `RegisterLocalisedBannerList` object.
    ///
    /// # Returns
    ///
    /// * `Ok(RegisterLocalisedBannerList)`: If the file was opened, read, and parsed successfully into `RegisterLocalisedBannerList`.
    /// * `Err(&'static str)`: If there is any failure in opening/reading the file or parsing the JSON, the appropriate error message is returned.
    ///
    /// # Errors
    ///
    /// This function will return an error in the following situations, but is not limited to just these cases:
    ///
    /// * The file could not be opened.
    /// * The file could not be read.
    /// * The JSON could not be parsed.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use crate::your_module::get_banner_register_localised;
    ///
    /// let result = get_banner_register_localised();
    /// match result {
    ///     Ok(banner) => {
    ///         // Do something with `banner`
    ///     }
    ///     Err(err) => {
    ///         eprintln!("Error: {}", err);
    ///     }
    /// }
    /// ```
    pub fn get_banner_register_localised() -> Result<RegisterLocalisedBannerList, &'static str> {
        let mut file = match File::open("./lang_file/command_register/general/banner.json") {
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
