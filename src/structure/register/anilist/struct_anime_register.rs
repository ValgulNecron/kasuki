use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RegisterLocalisedAnime {
    pub code: String,
    pub name: String,
    pub desc: String,
    pub option1: String,
    pub option1_desc: String,
}

type RegisterLocalisedAnimeList = HashMap<String, RegisterLocalisedAnime>;

impl RegisterLocalisedAnime {
    /// `get_anime_register_localised` is a function that returns a `Result`
    /// with either `RegisterLocalisedAnimeList` type data on success or an
    /// &'static str error string on failure.
    ///
    /// # Functionality
    ///
    /// The function attempts to open and read a json file located at
    /// './lang_file/command_register/anilist/anime.json'. It reads the file
    /// content into a `String` type variable `json`.
    ///
    /// The function then attempts to parse the `json` string into a
    /// `RegisterLocalisedAnimeList` type variable `data`.
    ///
    /// # Errors
    ///
    /// The function returns an error if it fails to open the file, read
    /// the file, or parse the json string. The specific actions that
    /// trigger these errors are encapsulated in `Result` returning
    /// expressions with the `?` operator.
    ///
    /// # Returns
    ///
    /// If the function is successful, it returns `Ok(data)` where `data`
    /// is of type `RegisterLocalisedAnimeList`. If an error occurs, it
    /// returns `Err("Failed to open file")` or `Err("Failed to read file")`
    /// or `Err("Failed to parse json.")` depending on the error.
    ///
    /// # Example
    ///
    /// ```
    /// use crate::RegisterLocalisedAnimeList; // Assume this is the right path
    ///
    /// let result = get_anime_register_localised();
    /// match result {
    ///     Ok(data) => println!("Successfully got data: {:?}", data),
    ///     Err(e) => println!("An error occurred: {}", e),
    /// }
    /// ```
    pub fn get_anime_register_localised() -> Result<RegisterLocalisedAnimeList, &'static str> {
        let mut file = match File::open("./lang_file/command_register/anilist/anime.json") {
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
