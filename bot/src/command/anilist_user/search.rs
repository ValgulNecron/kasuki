use std::error::Error;
use std::sync::Arc;

use moka::future::Cache;
use serenity::all::{CommandInteraction, Context};
use tokio::sync::RwLock;

use crate::command::anilist_user::anime::AnimeCommand;
use crate::command::anilist_user::character::CharacterCommand;
use crate::command::anilist_user::ln::LnCommand;
use crate::command::anilist_user::manga::MangaCommand;
use crate::command::anilist_user::staff::StaffCommand;
use crate::command::anilist_user::studio::StudioCommand;
use crate::command::anilist_user::user::UserCommand;
use crate::command::command_trait::{Command, SlashCommand};
use crate::config::Config;
use crate::error_management::error_dispatch;
use crate::helper::get_option::command::get_option_map_string;

pub struct SearchCommand {
    pub ctx: Context,
    pub command_interaction: CommandInteraction,
    pub config: Arc<Config>,
    pub anilist_cache: Arc<RwLock<Cache<String, String>>>,
}

impl Command for SearchCommand {
    fn get_ctx(&self) -> &Context {
        &self.ctx
    }

    fn get_command_interaction(&self) -> &CommandInteraction {
        &self.command_interaction
    }
}

impl SlashCommand for SearchCommand {
    async fn run_slash(&self) -> Result<(), Box<dyn Error>> {
        let ctx = &self.ctx;

        let command_interaction = &self.command_interaction;

        let config = &self.config;

        let anilist_cache = &self.anilist_cache;

        // Retrieve the type of AniList data to search for from the command interaction
        let map = get_option_map_string(command_interaction);

        let search_type = map
            .get(&String::from("type"))
            .ok_or(error_dispatch::Error::Option(String::from(
                "No type specified",
            )))?;

        // Execute the corresponding search function based on the specified type
        match search_type.as_str() {
            "anime" => {
                AnimeCommand {
                    ctx: ctx.clone(),
                    command_interaction: command_interaction.clone(),
                    config: config.clone(),
                    anilist_cache: anilist_cache.clone(),
                }
                .run_slash()
                .await
            }
            "character" => {
                CharacterCommand {
                    ctx: ctx.clone(),
                    command_interaction: command_interaction.clone(),
                    config: config.clone(),
                    anilist_cache: anilist_cache.clone(),
                }
                .run_slash()
                .await
            }
            "ln" => {
                LnCommand {
                    ctx: ctx.clone(),
                    command_interaction: command_interaction.clone(),
                    config: config.clone(),
                    anilist_cache: anilist_cache.clone(),
                }
                .run_slash()
                .await
            }
            "manga" => {
                MangaCommand {
                    ctx: ctx.clone(),
                    command_interaction: command_interaction.clone(),
                    config: config.clone(),
                    anilist_cache: anilist_cache.clone(),
                }
                .run_slash()
                .await
            }
            "staff" => {
                StaffCommand {
                    ctx: ctx.clone(),
                    command_interaction: command_interaction.clone(),
                    config: config.clone(),
                    anilist_cache: anilist_cache.clone(),
                }
                .run_slash()
                .await
            }
            "user" => {
                UserCommand {
                    ctx: ctx.clone(),
                    command_interaction: command_interaction.clone(),
                    config: config.clone(),
                    anilist_cache: anilist_cache.clone(),
                }
                .run_slash()
                .await
            }
            "studio" => {
                StudioCommand {
                    ctx: ctx.clone(),
                    command_interaction: command_interaction.clone(),
                    config: config.clone(),
                    anilist_cache: anilist_cache.clone(),
                }
                .run_slash()
                .await
            }
            // Return an error if the specified type is not one of the expected types
            _ => Err(Box::new(error_dispatch::Error::Option(String::from(
                "Type does not exist.",
            )))),
        }
    }
}
