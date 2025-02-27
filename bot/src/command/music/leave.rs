use crate::command::command_trait::{Command, Embed, EmbedContent, EmbedType, SlashCommand};
use crate::event_handler::BotData;
use anyhow::anyhow;
use serenity::all::{CommandInteraction, Context as SerenityContext};

pub struct  {
    pub ctx: SerenityContext,
    pub command_interaction: CommandInteraction,
}

impl Command for  {
    fn get_ctx(&self) -> &SerenityContext {
        &self.ctx
    }

    fn get_command_interaction(&self) -> &CommandInteraction {
        &self.command_interaction
    }
}

impl SlashCommand for {
    async fn run_slash(&self) -> anyhow::Result<()> {}
}