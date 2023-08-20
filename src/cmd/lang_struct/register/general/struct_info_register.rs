use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RegisterLocalisedInfo {
    pub code: String,
    pub name: String,
    pub desc: String,
}

type RegisterLocalisedInfoList = HashMap<String, RegisterLocalisedInfo>;

impl RegisterLocalisedInfo {
    pub fn get_info_register_localised() -> Result<RegisterLocalisedInfoList, &'static str> {
        let mut file = match File::open("lang_file/command_register/general/info.json") {
            Ok(file) => file,
            Err(_) => return Err("Failed to open file"),
        };
        let mut json = String::new();
        match file.read_to_string(&mut json) {
            Ok(_) => {}
            Err(_) => return Err("Failed to read file"),
        };
        return match serde_json::from_str(&json) {
            Ok(data) => Ok(data),
            Err(_) => Err("Failed to parse json."),
        };
    }
}