use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use serde::{Deserialize, Serialize};

use crate::error_enum::AppError;
use crate::error_enum::AppError::{
    LocalisationFileError, LocalisationParsingError, LocalisationReadError, NoLangageError,
};
use crate::function::general::get_guild_langage::get_guild_langage;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CreditLocalisedText {
    pub title: String,
    pub list: Vec<Credit>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Credit {
    pub text: String,
}

impl CreditLocalisedText {
    pub async fn get_credit_localised(guild_id: String) -> Result<CreditLocalisedText, AppError> {
        let mut file = File::open("./lang_file/embed/general/credit.json")
            .map_err(|_| LocalisationFileError(String::from("File credit.json not found.")))?;

        let mut json = String::new();
        file.read_to_string(&mut json)
            .map_err(|_| LocalisationReadError(String::from("File credit.json can't be read.")))?;

        let json_data: HashMap<String, CreditLocalisedText> = serde_json::from_str(&json)
            .map_err(|_| LocalisationParsingError(String::from("Failing to parse credit.json.")))?;

        let lang_choice = get_guild_langage(&guild_id).await;

        let avatar_localised_text = json_data
            .get(lang_choice.as_str())
            .ok_or(NoLangageError(String::from("not found")))?;

        Ok(avatar_localised_text.clone())
    }
}
