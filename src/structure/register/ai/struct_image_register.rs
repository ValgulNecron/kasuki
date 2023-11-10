use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ImageRegister {
    pub code: String,
    pub name: String,
    pub desc: String,
    pub option1: String,
    pub option1_desc: String,
}

type RegisterLocalisedImageList = HashMap<String, ImageRegister>;

impl ImageRegister {
    /// Retrieves the register of localised images from a JSON file.
    ///
    /// The function attempts to open the file located at `./lang_file/command_register/ai/image.json`
    /// and read its contents as a JSON string. It then attempts to parse the JSON string into a
    /// `RegisterLocalisedImageList` object using the `serde_json::from_str` function. If successful,
    /// the parsed object is returned as a `Result::Ok`. If any of the file operations or parsing fails,
    /// an error message is returned as a `Result::Err`.
    ///
    /// # Errors
    ///
    /// - If the file fails to open, an error message `"Failed to open file"` is returned.
    /// - If the file fails to read, an error message `"Failed to read file"` is returned.
    /// - If the JSON fails to parse, an error message `"Failed to parse json."` is returned.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use std::fs::File;
    /// use std::io::Read;
    /// use serde_json::Result;
    ///
    /// #[derive(serde::Deserialize)]
    /// struct RegisterLocalisedImageList {
    ///     // struct fields...
    /// }
    ///
    /// pub fn get_image_register_localised() -> Result<RegisterLocalisedImageList, &'static str> {
    ///     let mut file = match File::open("./lang_file/command_register/ai/image.json") {
    ///         Ok(file) => file,
    ///         Err(_) => return Err("Failed to open file"),
    ///     };
    ///     let mut json = String::new();
    ///     match file.read_to_string(&mut json) {
    ///         Ok(_) => {}
    ///         Err(_) => return Err("Failed to read file"),
    ///     };
    ///     match serde_json::from_str(&json) {
    ///         Ok(data) => Ok(data),
    ///         Err(_) => Err("Failed to parse json."),
    ///     }
    /// }
    /// ```
    pub fn get_image_register_localised() -> Result<RegisterLocalisedImageList, &'static str> {
        let mut file = match File::open("./lang_file/command_register/ai/image.json") {
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
