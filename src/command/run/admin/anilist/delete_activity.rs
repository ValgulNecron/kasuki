use std::sync::Arc;

use moka::future::Cache;
use serenity::all::CreateInteractionResponse::Defer;
use serenity::all::{
    CommandInteraction, Context, CreateInteractionResponseFollowup,
    CreateInteractionResponseMessage,
};
use tokio::sync::RwLock;
use tracing::trace;

use crate::command::run::admin::anilist::add_activity::{
    get_minimal_anime_by_id, get_minimal_anime_by_search, get_name,
};
use crate::config::Config;
use crate::database::manage::dispatcher::data_dispatch::remove_data_activity_status;
use crate::helper::create_default_embed::get_default_embed;
use crate::helper::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use crate::helper::get_option::subcommand_group::get_option_map_string_subcommand_group;
use crate::structure::message::admin::anilist::delete_activity::load_localization_delete_activity;

/// This asynchronous function runs the command interaction for deleting an anime activity.
///
/// It first retrieves the anime name from the command interaction options.
/// It then creates a deferred response to the command interaction.
///
/// It retrieves the anime data from AniList using either the anime ID or name, depending on whether the anime option can be parsed as an integer.
/// It retrieves the anime ID from the anime data.
///
/// It removes the activity for the anime and server from the database.
///
/// It retrieves the anime data again from AniList using the anime ID.
/// It retrieves the anime name and cover image from the anime data.
///
/// It creates an embed for the followup message, including the anime name, cover image, and a success message.
///
/// Finally, it sends a followup message with the embed.
///
/// # Arguments
///
/// * `ctx` - The context in which this function is being called.
/// * `command_interaction` - The command interaction that triggered this function.
///
/// # Returns
///
/// A `Result` indicating whether the function executed successfully. If an error occurred, it contains an `AppError`.
pub async fn run(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    config: Arc<Config>,
    anilist_cache: Arc<RwLock<Cache<String, String>>>,
) -> Result<(), AppError> {
    let db_type = config.bot.config.db_type.clone();
    let map = get_option_map_string_subcommand_group(command_interaction);
    let anime = map
        .get(&String::from("anime_name"))
        .cloned()
        .unwrap_or(String::new());

    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };

    let delete_activity_localised_text =
        load_localization_delete_activity(guild_id.clone(), db_type.clone()).await?;
    let builder_message = Defer(CreateInteractionResponseMessage::new());

    command_interaction
        .create_response(&ctx.http, builder_message)
        .await
        .map_err(|e| {
            AppError::new(
                format!("Error while sending the command {}", e),
                ErrorType::Command,
                ErrorResponseType::Message,
            )
        })?;
    trace!(anime);
    let media = if anime.parse::<i32>().is_ok() {
        get_minimal_anime_by_id(anime.parse::<i32>().unwrap(), anilist_cache).await?
    } else {
        get_minimal_anime_by_search(anime.as_str(), anilist_cache).await?
    };

    let anime_id = media.id;
    remove_activity(guild_id.as_str(), &anime_id, db_type).await?;

    let title = media.title.unwrap();
    let anime_name = get_name(title);
    let builder_embed = get_default_embed(None)
        .title(&delete_activity_localised_text.success)
        .url(format!("https://anilist.co/anime/{}", media.id))
        .description(
            delete_activity_localised_text
                .success_desc
                .replace("$anime$", anime_name.as_str()),
        );

    let builder_message = CreateInteractionResponseFollowup::new().embed(builder_embed);

    command_interaction
        .create_followup(&ctx.http, builder_message)
        .await
        .map_err(|e| {
            AppError::new(
                format!("Error while sending the command {}", e),
                ErrorType::Command,
                ErrorResponseType::Followup,
            )
        })?;
    Ok(())
}

/// This asynchronous function removes an activity for a given anime and server from the database.
///
/// # Arguments
///
/// * `guild_id` - The ID of the server from which to remove the activity.
/// * `anime_id` - The ID of the anime for which to remove the activity.
///
/// # Returns
///
/// A `Result` indicating whether the function executed successfully. If an error occurred, it contains an `AppError`.
async fn remove_activity(guild_id: &str, anime_id: &i32, db_type: String) -> Result<(), AppError> {
    remove_data_activity_status(guild_id.to_owned(), anime_id.to_string(), db_type).await
}
