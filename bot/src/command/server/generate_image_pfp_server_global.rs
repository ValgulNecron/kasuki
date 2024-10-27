use std::sync::Arc;

use crate::command::command_trait::{Command, SlashCommand};
use crate::command::server::generate_image_pfp_server::send_embed;
use crate::config::Config;
use crate::event_handler::BotData;
use anyhow::Result;
use serenity::all::{CommandInteraction, Context as SerenityContext};

pub struct GenerateGlobalImagePfPCommand {
    pub ctx: SerenityContext,
    pub command_interaction: CommandInteraction,
}

impl Command for GenerateGlobalImagePfPCommand {
    fn get_ctx(&self) -> &SerenityContext {
        &self.ctx
    }

    fn get_command_interaction(&self) -> &CommandInteraction {
        &self.command_interaction
    }
}

impl SlashCommand for GenerateGlobalImagePfPCommand {
    async fn run_slash(&self) -> Result<()> {
        let ctx = self.get_ctx();
        let bot_data = ctx.data::<BotData>().clone();
        init(
            &self.ctx,
            &self.command_interaction,
            bot_data.config.clone(),
        )
        .await
    }
}

pub async fn init(
    ctx: &SerenityContext,
    command_interaction: &CommandInteraction,
    config: Arc<Config>,
) -> Result<()> {
    send_embed(ctx, command_interaction, "global", config.db.clone()).await
}
