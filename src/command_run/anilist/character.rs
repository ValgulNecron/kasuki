use serenity::all::{CommandDataOption, CommandInteraction, Context};

use crate::anilist_struct::run::character::{send_embed, CharacterWrapper};
use crate::error_enum::AppError;

pub async fn run(
    options: &[CommandDataOption],
    ctx: &Context,
    command: &CommandInteraction,
) -> Result<(), AppError> {
    let mut value = String::new();
    for option_data in options {
        if option_data.name.as_str() != "type" {
            let option_value = option_data.value.as_str().unwrap();
            value = option_value.to_string().clone()
        }
    }

    let data: CharacterWrapper = if value.parse::<i32>().is_ok() {
        CharacterWrapper::new_character_by_id(value.parse().unwrap()).await?
    } else {
        CharacterWrapper::new_character_by_search(&value).await?
    };

    send_embed(ctx, command, data).await
}
