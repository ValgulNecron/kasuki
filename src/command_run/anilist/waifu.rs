use serenity::all::{CommandInteraction, Context};

use crate::anilist_struct::run::character::{send_embed, CharacterWrapper};
use crate::error_management::error_enum::AppError;

pub async fn run(
    ctx: &Context,
    command_interaction: &CommandInteraction,
) -> Result<(), AppError> {
    let data: CharacterWrapper = CharacterWrapper::new_character_by_id(156323)
        .await?;

    send_embed(ctx, command_interaction, data).await
}
