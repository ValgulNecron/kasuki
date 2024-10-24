use crate::event_handler::Handler;
use anyhow::{Context, Error, Result};
use serenity::all::{CommandInteraction, Context as SerenityContext};
pub async fn dispatch_message_command(
    _ctx: &SerenityContext,
    command_interaction: &CommandInteraction,
    _self_handler: &Handler,
) -> Result<()> {
    match command_interaction.data.name.as_str() {
        _ => Err(Error::from("Unknown command")),
    }
}
