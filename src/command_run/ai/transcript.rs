use crate::constant::OPTION_ERROR;
use crate::error_enum::AppError;
use crate::error_enum::AppError::NoCommandOption;
use serenity::all::{
    Attachment, CommandDataOption, CommandDataOptionValue, CommandInteraction, Context,
    ResolvedOption, ResolvedValue,
};

pub async fn run(
    options: &[ResolvedOption<'_>],
    ctx: &Context,
    command: &CommandInteraction,
) -> Result<(), AppError> {
    let mut prompt: String = String::new();
    let mut lang: String = String::new();
    let mut attachement: Option<Attachment> = None;
    for option in options.to_owned() {
        if option.name == "lang_struct" {
            let resolved = option.value.clone();
            if let ResolvedValue::String(lang_option) = resolved {
                lang = String::from(lang_option)
            }
        }
        if option.name == "prompt" {
            let resolved = option.value.clone();
            if let ResolvedValue::String(prompt_option) = resolved {
                prompt = String::from(prompt_option)
            }
        }
        if option.name == "video" {
            if let ResolvedOption {
                value: ResolvedValue::Attachment(attachment_option),
                ..
            } = option
            {
                attachement = Some(attachment_option.to_owned())
            } else {
                return Err(NoCommandOption(String::from(
                    "The command contain no option.",
                )));
            }
        }
    }

    Ok(())
}
