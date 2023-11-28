use crate::error_enum::AppError;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

#[derive(Debug, Serialize, Deserialize)]
pub struct Credit {
    pub desc: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Locale {
    pub title: String,
    pub credits: Vec<Credit>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Localization {
    pub en: Locale,
    pub fr: Locale,
    pub jp: Locale,
    pub de: Locale,
}

pub fn load_localization() -> Result<Localization, AppError> {
    let file = File::open("./json/command/credit.json")?;
    let mut reader = BufReader::new(file);
    let mut json_string = String::new();
    reader.read_to_string(&mut json_string)?;
    let localization: Localization = serde_json::from_str(&json_string)?;

    Ok(localization)
}
