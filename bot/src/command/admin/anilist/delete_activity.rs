use std::error::Error;
use std::sync::Arc;

use crate::command::admin::anilist::add_activity::{get_minimal_anime_media, get_name};
use crate::command::command_trait::{Command, Embed, EmbedType, SlashCommand};
use crate::config::{BotConfigDetails, Config};
use crate::get_url;
use crate::helper::error_management::error_dispatch;
use crate::helper::get_option::subcommand_group::get_option_map_string_subcommand_group;
use crate::structure::database::prelude::ActivityData;
use crate::structure::message::admin::anilist::delete_activity::load_localization_delete_activity;
use moka::future::Cache;
use sea_orm::ColumnTrait;
use sea_orm::{EntityTrait, ModelTrait, QueryFilter};
use serenity::all::{CommandInteraction, Context};
use tokio::sync::RwLock;

pub struct DeleteActivityCommand {
    pub ctx: Context,
    pub command_interaction: CommandInteraction,
    pub config: Arc<Config>,
    pub anilist_cache: Arc<RwLock<Cache<String, String>>>,
}

impl Command for DeleteActivityCommand {
    fn get_ctx(&self) -> &Context {
        &self.ctx
    }

    fn get_command_interaction(&self) -> &CommandInteraction {
        &self.command_interaction
    }
}

impl SlashCommand for DeleteActivityCommand {
    async fn run_slash(&self) -> Result<(), Box<dyn Error>> {
        let anilist_cache = self.anilist_cache.clone();
        let command_interaction = self.command_interaction.clone();
        let config = self.config.clone();
        let ctx = self.ctx.clone();
        let map = get_option_map_string_subcommand_group(&command_interaction);
        let anime = map
            .get(&String::from("anime_name"))
            .cloned()
            .unwrap_or(String::new());

        let guild_id = match command_interaction.guild_id {
            Some(id) => id.to_string(),
            None => String::from("1"),
        };
        self.defer().await?;

        let delete_activity_localised_text =
            load_localization_delete_activity(guild_id.clone(), config.bot.config.clone()).await?;

        let media = get_minimal_anime_media(anime.to_string(), anilist_cache).await?;

        let anime_id = media.id;
        remove_activity(guild_id.as_str(), &anime_id, config.bot.config.clone()).await?;

        let title = media.title.ok_or(error_dispatch::Error::Option(format!(
            "Anime with id {} not found",
            anime_id
        )))?;
        let anime_name = get_name(title);

        let url = format!("https://anilist.co/anime/{}", media.id);

        self.send_embed(
            Vec::new(),
            None,
            delete_activity_localised_text.success.clone(),
            delete_activity_localised_text
                .success_desc
                .replace("$anime$", anime_name.as_str()),
            None,
            Some(url),
            EmbedType::Followup,
            None,
        )
        .await?;
        Ok(())
    }
}

async fn remove_activity(
    guild_id: &str,
    anime_id: &i32,
    db_config: BotConfigDetails,
) -> Result<(), Box<dyn Error>> {
    let connection = sea_orm::Database::connect(get_url(db_config.clone())).await?;
    let activity = ActivityData::find()
        .filter(crate::structure::database::activity_data::Column::ServerId.eq(guild_id))
        .filter(crate::structure::database::activity_data::Column::AnimeId.eq(anime_id.to_string()))
        .one(&connection)
        .await?
        .ok_or(error_dispatch::Error::Option(format!(
            "Anime with id {} not found",
            anime_id
        )))?;
    activity.delete(&connection).await?;
    Ok(())
}
