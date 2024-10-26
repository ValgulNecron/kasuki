use anyhow::{anyhow, Result};
use std::sync::Arc;

use moka::future::Cache;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use small_fixed_array::FixedString;
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
use crate::event_handler::BotData;
use crate::helper::get_option::command::get_option_map_string;

pub struct SearchCommand {
    pub ctx: SerenityContext,
    pub command_interaction: CommandInteraction,
}

impl Command for SearchCommand {
    fn get_ctx(&self) -> &SerenityContext {
        &self.ctx
    }

    fn get_command_interaction(&self) -> &CommandInteraction {
        &self.command_interaction
    }
}

impl SlashCommand for SearchCommand {
    async fn run_slash(&self) -> Result<()> {
        let ctx = self.get_ctx();
        let bot_data = ctx.data::<BotData>().clone();
        let command_interaction = &self.command_interaction;

        let config = &bot_data.config;

        let anilist_cache = &bot_data.anilist_cache;

        // Retrieve the type of AniList data to search for from the command interaction
        let map = get_option_map_string(command_interaction);

        let search_type = map
            .get(&FixedString::from_str_trunc("type"))
            .ok_or(anyhow!("No type specified"))?;

        // Execute the corresponding search function based on the specified type
        match search_type.as_str() {
            "anime" => {
                AnimeCommand {
                    ctx: ctx.clone(),
                    command_interaction: command_interaction.clone(),
                }
                .run_slash()
                .await
            }
            "character" => {
                CharacterCommand {
                    ctx: ctx.clone(),
                    command_interaction: command_interaction.clone(),
                }
                .run_slash()
                .await
            }
            "ln" => {
                LnCommand {
                    ctx: ctx.clone(),
                    command_interaction: command_interaction.clone(),
                }
                .run_slash()
                .await
            }
            "manga" => {
                MangaCommand {
                    ctx: ctx.clone(),
                    command_interaction: command_interaction.clone(),
                }
                .run_slash()
                .await
            }
            "staff" => {
                StaffCommand {
                    ctx: ctx.clone(),
                    command_interaction: command_interaction.clone(),
                }
                .run_slash()
                .await
            }
            "user" => {
                UserCommand {
                    ctx: ctx.clone(),
                    command_interaction: command_interaction.clone(),
                }
                .run_slash()
                .await
            }
            "studio" => {
                StudioCommand {
                    ctx: ctx.clone(),
                    command_interaction: command_interaction.clone(),
                }
                .run_slash()
                .await
            }
            // Return an error if the specified type is not one of the expected types
            _ => Err(anyhow!("Type does not exist.")),
        }
    }
}
