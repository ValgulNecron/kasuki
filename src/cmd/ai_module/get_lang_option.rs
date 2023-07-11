use serenity::model::prelude::application_command::{CommandDataOption, CommandDataOptionValue};

pub fn get_lang_option(option: CommandDataOption) -> String {
    let mut lang = "en".to_string();
    if option.name == "lang" {
        let resolved = option.resolved.as_ref().unwrap();
        if let CommandDataOptionValue::String(lang_option) = resolved {
            lang = lang_option.clone()
        } else {
            lang = "en".to_string()
        }
    }
    return lang;
}
