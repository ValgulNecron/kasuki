use crate::command_autocomplete::anilist::anime;
use serenity::all::{CommandInteraction, Context, Interaction};

pub async fn autocomplete_dispatching(
    ctx: Context,
    interaction: Interaction,
    command: CommandInteraction,
) {
    match command.data.name.as_str() {
        "anime" => anime::autocomplete(ctx, interaction, command).await,
        _ => {}
    }
}
