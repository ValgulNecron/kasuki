use anyhow::Result;
use std::sync::Arc;

use moka::future::Cache;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use tokio::sync::RwLock;

use crate::command::anilist_user::character::get_character_by_id;
use crate::command::command_trait::{Command, SlashCommand};
use crate::config::Config;
use crate::event_handler::BotData;
use crate::structure::run::anilist::character::send_embed;

pub struct WaifuCommand {
    pub ctx: SerenityContext,
    pub command_interaction: CommandInteraction,
}

impl Command for WaifuCommand {
    fn get_ctx(&self) -> &SerenityContext {
        &self.ctx
    }

    fn get_command_interaction(&self) -> &CommandInteraction {
        &self.command_interaction
    }
}

impl SlashCommand for WaifuCommand {
    async fn run_slash(&self) -> Result<()> {
        let ctx = self.get_ctx();
        let bot_data = ctx.data::<BotData>().clone();
        let command_interaction = &self.command_interaction;

        let config = bot_data.config.clone();

        let anilist_cache = bot_data.anilist_cache.clone();

        // Execute the corresponding search function based on the specified type
        // Fetch the data of the character with ID 156323 from AniList
        let value = 156323;

        let data = get_character_by_id(value, anilist_cache).await?;

        // Send the character's data as a response to the command interaction
        send_embed(ctx, command_interaction, data, config.db.clone()).await
    }
}
