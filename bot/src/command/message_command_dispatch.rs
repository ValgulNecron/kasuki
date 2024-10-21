use crate::event_handler::Handler;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use anyhow::{Context, Error, Result};
pub async fn dispatch_message_command(
    _ctx: &SerenityContext,
    command_interaction: &CommandInteraction,
    _self_handler: &Handler,
) -> Result<()> {
    match command_interaction.data.name.as_str() {
        _ => Err(Error::from(
            "Unknown command",
        )),
    }
}
