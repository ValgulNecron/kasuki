use std::error::Error;
use std::sync::Arc;

use moka::future::Cache;
use serenity::all::{
    CommandInteraction, Context, CreateInteractionResponse, CreateInteractionResponseMessage,
};
use tokio::sync::RwLock;

use crate::config::Config;
use crate::helper::create_default_embed::get_default_embed;
use crate::helper::error_management::error_enum::ResponseError;
use crate::helper::vndbapi::stats::get_stats;
use crate::structure::message::vn::stats::load_localization_stats;

pub async fn run(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    config: Arc<Config>,
    vndb_cache: Arc<RwLock<Cache<String, String>>>,
) -> Result<(), Box<dyn Error>> {
    let db_type = config.bot.config.db_type.clone();
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };
    let stats_localised =
        load_localization_stats(guild_id, db_type, config.bot.config.clone()).await?;

    let stats = get_stats(vndb_cache).await?;

    let fields = vec![
        (stats_localised.chars.clone(), stats.chars.to_string(), true),
        (
            stats_localised.producer.clone(),
            stats.producers.to_string(),
            true,
        ),
        (
            stats_localised.release.clone(),
            stats.releases.to_string(),
            true,
        ),
        (stats_localised.staff.clone(), stats.staff.to_string(), true),
        (stats_localised.staff.clone(), stats.staff.to_string(), true),
        (stats_localised.tags.clone(), stats.tags.to_string(), true),
        (
            stats_localised.traits.clone(),
            stats.traits.to_string(),
            true,
        ),
        (stats_localised.vns.clone(), stats.vn.to_string(), true),
        (stats_localised.api.clone(), String::from("VNDB API"), true),
    ];
    let builder_embed = get_default_embed(None)
        .title(stats_localised.title)
        .fields(fields);
    let builder_message = CreateInteractionResponseMessage::new().embed(builder_embed);
    let builder = CreateInteractionResponse::Message(builder_message);
    command_interaction
        .create_response(&ctx.http, builder)
        .await
        .map_err(|e| ResponseError::Sending(format!("{:#?}", e)))?;
    Ok(())
}
