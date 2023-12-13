use crate::anilist_struct::run::character::{send_embed, CharacterWrapper};
use crate::error_enum::AppError;
use serenity::all::{CommandInteraction, Context};

pub async fn run(ctx: &Context, command: &CommandInteraction) -> Result<(), AppError> {
    let data: CharacterWrapper = CharacterWrapper::new_character_by_id(156323).await?;

    send_embed(ctx, command, data).await
}
