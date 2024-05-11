use serenity::all::CreateInteractionResponse::Defer;
use serenity::all::{
    CommandInteraction, Context, CreateInteractionResponseFollowup,
    CreateInteractionResponseMessage,
};

use crate::anilist_struct::run::minimal_anime::MinimalAnimeWrapper;
use crate::command::run::admin::anilist::add_activity::get_name;
use crate::helper::create_normalise_embed::get_default_embed;
use crate::helper::get_option::subcommand::get_option_map_string_subcommand;
use crate::database::manage::dispatcher::data_dispatch::remove_data_activity_status;
use crate::helper::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
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
pub async fn run(ctx: &Context, command_interaction: &CommandInteraction) -> Result<(), AppError> {
    let map = get_option_map_string_subcommand(command_interaction);
    let anime = map
        .get(&String::from("anime_name"))
        .cloned()
        .unwrap_or(String::new());

    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };

    let delete_activity_localised_text = load_localization_delete_activity(guild_id).await?;
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };
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
    let anime_id = if anime.parse::<i32>().is_ok() {
        anime.parse().unwrap()
    } else {
        MinimalAnimeWrapper::new_minimal_anime_by_search(anime.to_string())
            .await?
            .data
            .media
            .id
    };

    remove_activity(&guild_id, &anime_id).await?;

    let data = MinimalAnimeWrapper::new_minimal_anime_by_id(anime_id.to_string()).await?;
    let media = data.data.media;
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
async fn remove_activity(guild_id: &str, anime_id: &i32) -> Result<(), AppError> {
    remove_data_activity_status(guild_id.to_owned(), anime_id.to_string()).await
}
