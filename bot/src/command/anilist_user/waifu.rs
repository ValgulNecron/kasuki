use std::error::Error;
use std::sync::Arc;

use moka::future::Cache;
use serenity::all::{CommandInteraction, Context};
use tokio::sync::RwLock;

use crate::command::anilist_user::character::get_character_by_id;
use crate::command::command_trait::{Command, SlashCommand};
use crate::config::Config;
use crate::structure::run::anilist::character::send_embed;

pub struct WaifuCommand {
    pub ctx: Context,
    pub command_interaction: CommandInteraction,
    pub config: Arc<Config>,
    pub anilist_cache: Arc<RwLock<Cache<String, String>>>,
}

impl Command for WaifuCommand {
    fn get_command_interaction(&self) -> &CommandInteraction {
        &self.command_interaction
    }

    fn get_ctx(&self) -> &Context {
        &self.ctx
    }
}

impl SlashCommand for WaifuCommand {
    async fn run_slash(&self) -> Result<(), Box<dyn Error>> {
        let ctx = &self.ctx;
        let command_interaction = &self.command_interaction;
        let config = self.config.clone();
        let anilist_cache = self.anilist_cache.clone();
        // Execute the corresponding search function based on the specified type
        // Fetch the data of the character with ID 156323 from AniList
        let value = 156323;
        let data = get_character_by_id(value, anilist_cache).await?;
        // Send the character's data as a response to the command interaction
        send_embed(ctx, command_interaction, data, config.bot.config.clone()).await
    }
}
