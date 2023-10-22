use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RegisterLocalisedCredit {
    pub code: String,
    pub name: String,
    pub desc: String,
}

type RegisterLocalisedCreditList = HashMap<String, RegisterLocalisedCredit>;

impl RegisterLocalisedCredit {
    pub fn get_credit_register_localised() -> Result<RegisterLocalisedCreditList, &'static str> {
        let mut file = match File::open("lang_file/command_register/general/credit.json") {
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
