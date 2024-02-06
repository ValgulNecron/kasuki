use crate::common::get_guild_lang::get_guild_langage;
use crate::error_enum::AppError;
use crate::error_enum::AppError::NewMemberError;
use crate::error_enum::NewMemberError::{
    NewMemberLocalisationFileError, NewMemberLocalisationParsingError,
    NewMemberLocalisationReadError, NewMemberNoLanguageError,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct NewMemberLocalised {
    pub welcome: String,
}

pub async fn load_localization_new_member(
    guild_id: String,
) -> Result<NewMemberLocalised, AppError> {
    let mut file = File::open("json/message/new_member.json").map_err(|e| {
        NewMemberError(NewMemberLocalisationFileError(format!(
            "File new_member.json not found. {}",
            e
        )))
    })?;

    let mut json = String::new();
    file.read_to_string(&mut json).map_err(|e| {
        NewMemberError(NewMemberLocalisationReadError(format!(
            "File new_member.json can't be read. {}",
            e
        )))
    })?;

    let json_data: HashMap<String, NewMemberLocalised> =
        serde_json::from_str(&json).map_err(|e| {
            NewMemberError(NewMemberLocalisationParsingError(format!(
                "Failing to parse new_member.json. {}",
                e
            )))
        })?;

    let lang_choice = get_guild_langage(guild_id).await;

    let new_member_localised_text =
        json_data
            .get(lang_choice.as_str())
            .ok_or(NewMemberError(NewMemberNoLanguageError(String::from(
                "not found",
            ))))?;

    Ok(new_member_localised_text.clone())
}
