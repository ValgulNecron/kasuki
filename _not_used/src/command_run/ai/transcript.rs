use crate::constant::OPTION_ERROR;
use crate::error_enum::AppError;
use crate::error_enum::AppError::NoCommandOption;
use serenity::all::{
    Attachment, CommandDataOption, CommandDataOptionValue, CommandInteraction, Context,
};

pub async fn run(
    options: &[CommandDataOption],
    ctx: &Context,
    command: &CommandInteraction,
) -> Result<(), AppError> {
    let mut prompt: String = String::new();
    let mut lang: String = String::new();
    let mut attachement: Option<Attachment> = None;
    for option in options {
        if option.name == "lang_struct" {
            let resolved = &option.value;
            if let CommandDataOptionValue::String(lang_option) = resolved {
                lang = lang_option.clone()
            }
        }
        if option.name == "prompt" {
            let resolved = &option.value;
            if let CommandDataOptionValue::String(prompt_option) = resolved {
                prompt = prompt_option.clone()
            }
        }
        if option.name == "video" {
            let resolved = &option.value;
            if let CommandDataOptionValue::Attachment(attachement_option) = resolved {
                // attachement = ;
            } else {
                return Err(NoCommandOption(String::from(
                    "The command contain no option.",
                )));
            }
        }
    }

    Ok(())
}
