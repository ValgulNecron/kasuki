use log::trace;
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
pub struct ModuleLocalisedText {
    pub on: String,
    pub off: String,
}

impl ModuleLocalisedText {
    pub async fn get_module_localised(guild_id: &String) -> Result<ModuleLocalisedText, AppError> {
        let mut file =
            File::open("./lang_file/embed/general/module_activation.json").map_err(|_| {
                LocalisationFileError(String::from("File module_activation.json not found."))
            })?;

        let mut json = String::new();
        file.read_to_string(&mut json).map_err(|_| {
            LocalisationReadError(String::from("File module_activation.json can't be read."))
        })?;

        let json_data: HashMap<String, ModuleLocalisedText> =
            serde_json::from_str(&json).map_err(|_| {
                LocalisationParsingError(String::from("Failing to parse module_activation.json."))
            })?;

        let lang_choice = get_guild_langage(guild_id).await;
        trace!("{}", lang_choice);

        let avatar_localised_text = json_data
            .get(lang_choice.as_str())
            .ok_or(NoLangageError(String::from("not found")))?;
        trace!("{:?}", avatar_localised_text);
        Ok(avatar_localised_text.clone())
    }
}
