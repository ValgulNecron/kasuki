use anyhow::{anyhow, Result};
use serenity::all::{CommandInteraction, Context as SerenityContext};
pub async fn dispatch_message_command(
    _ctx: &SerenityContext,
    command_interaction: &CommandInteraction,
) -> Result<()> {
    match command_interaction.data.name.as_str() {
        _ => Err(anyhow!("Unknown command")),
    }
}
