use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RegisterLocalisedTranslation {
    pub code: String,
    pub name: String,
    pub desc: String,
    pub option1: String,
    pub option1_desc: String,
    pub option2: String,
    pub option2_desc: String,
}

type RegisterLocalisedTranslationList = HashMap<String, RegisterLocalisedTranslation>;

impl RegisterLocalisedTranslation {
    /// Reads a JSON translation file and returns a deserialized `RegisterLocalisedTranslationList`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::fs::File;
    /// use std::io::prelude::*;
    /// use serde_json::Result;
    ///
    /// pub fn get_translation_register_localised(
    /// ) -> Result<RegisterLocalisedTranslationList, &'static str> {
    ///     let mut file = match File::open("./lang_file/command_register/ai/translation.json") {
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
    ///
    /// # Errors
    ///
    /// Returns an `Err` containing an error message in case of any failure during file operations or JSON parsing.
    pub fn get_translation_register_localised(
    ) -> Result<RegisterLocalisedTranslationList, &'static str> {
        let mut file = match File::open("./lang_file/command_register/ai/translation.json") {
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
