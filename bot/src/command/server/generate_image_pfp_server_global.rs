use std::error::Error;
use std::sync::Arc;

use crate::command::command_trait::{Command, SlashCommand};
use crate::command::server::generate_image_pfp_server::send_embed;
use crate::config::Config;
use serenity::all::{CommandInteraction, Context};

pub struct GenerateGlobalImagePfPCommand {
    pub ctx: Context,
    pub command_interaction: CommandInteraction,
    pub config: Arc<Config>,
}

impl Command for GenerateGlobalImagePfPCommand {
    fn get_ctx(&self) -> &Context {
        &self.ctx
    }

    fn get_command_interaction(&self) -> &CommandInteraction {
        &self.command_interaction
    }
}

impl SlashCommand for GenerateGlobalImagePfPCommand {
    async fn run_slash(&self) -> Result<(), Box<dyn Error>> {
        init(&self.ctx, &self.command_interaction, self.config.clone()).await
    }
}
pub async fn init(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    config: Arc<Config>,
) -> Result<(), Box<dyn Error>> {
    send_embed(
        ctx,
        command_interaction,
        "global",
        config.bot.config.clone(),
    )
    .await
}
