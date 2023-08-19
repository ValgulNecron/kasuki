use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AvailableLang {
    pub lang: String,
}

type AvailableLangList = HashMap<String, AvailableLang>;

impl AvailableLang {
    pub fn get_available_lang() -> Result<AvailableLangList, &'static str> {
        let mut file =
            match File::open("lang_file/available_lang.json") {
            Ok(file) => file,
                Err(_) => return Err("Failed to open file"),
        };
        let mut json = String::new();
        match file.read_to_string(&mut json){
            Ok(_) => {},
            Err(_) => return Err("Failed to read file"),
        };
        return match serde_json::from_str(&json) {
            Ok(data) => Ok(data),
            Err(_) => Err("Failed to parse json."),
        }
    }
}