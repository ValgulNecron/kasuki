use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RegisterLocalisedUser {
    pub code: String,
    pub name: String,
    pub desc: String,
    pub option1: String,
    pub option1_desc: String,
}

type RegisterLocalisedUserList = HashMap<String, RegisterLocalisedUser>;

impl RegisterLocalisedUser {
    pub fn get_user_register_localised() -> Result<RegisterLocalisedUserList, &'static str> {
        let mut file = match File::open("lang_file/command_register/anilist/user.json") {
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
