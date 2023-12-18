use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;

use serde::{Deserialize, Serialize};
use tracing::trace;

use crate::common::get_guild_lang::get_guild_langage;
use crate::error_enum::AppError;
use crate::error_enum::AppError::{
    LocalisationFileError, LocalisationParsingError, LocalisationReadError, NoLangageError,
};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StaffLocalised {
    pub field1_title: String,
    pub field2_title: String,
    pub desc: String,
    pub date_of_birth: String,
    pub date_of_death: String,
}

pub async fn load_localization_staff(guild_id: String) -> Result<StaffLocalised, AppError> {
    let mut file = File::open("json/message/anilist/staff.json")
        .map_err(|_| LocalisationFileError(String::from("File staff.json not found.")))?;

    let mut json = String::new();
    file.read_to_string(&mut json)
        .map_err(|_| LocalisationReadError(String::from("File staff.json can't be read.")))?;

    let json_data: HashMap<String, StaffLocalised> = serde_json::from_str(&json)
        .map_err(|_| LocalisationParsingError(String::from("Failing to parse staff.json.")))?;

    trace!("{}", guild_id);
    trace!("{}", guild_id != String::from("0"));

    let lang_choice = get_guild_langage(guild_id).await;

    let staff_localised_text = json_data
        .get(lang_choice.as_str())
        .ok_or(NoLangageError(String::from("not found")))?;

    Ok(staff_localised_text.clone())
}