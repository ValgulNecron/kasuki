use serenity::all::{CommandInteraction, Context};

use crate::anilist_struct::run::character::{send_embed, CharacterWrapper};
use crate::error_management::command_error::CommandError;
use crate::error_management::interaction_error::InteractionError;

pub async fn run(
    ctx: &Context,
    command_interaction: &CommandInteraction,
) -> Result<(), InteractionError> {
    let data: CharacterWrapper = CharacterWrapper::new_character_by_id(156323)
        .await
        .map_err(|e| CommandError::WebRequestError(e))?;

    send_embed(ctx, command_interaction, data).await
}
