use crate::background_task::update_random_stats::update_random_stats;
use chrono::Utc;
use cynic::{GraphQlResponse, QueryBuilder};
use rand::{thread_rng, Rng};
use serenity::all::CreateInteractionResponse::Defer;
use serenity::all::{
    CommandInteraction, Context, CreateInteractionResponseFollowup,
    CreateInteractionResponseMessage,
};

use crate::cache::cache_struct::random_cache::RandomCache;
use crate::cache::manage::cache_dispatch::{get_database_random_cache, set_database_random_cache};
use crate::helper::convert_flavored_markdown::convert_anilist_flavored_to_discord_flavored_markdown;
use crate::helper::create_default_embed::get_default_embed;
use crate::helper::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use crate::helper::get_option::subcommand::get_option_map_string_subcommand;
use crate::helper::make_graphql_cached::make_request_anilist;
use crate::helper::trimer::trim;
use crate::structure::message::anilist_user::random::{load_localization_random, RandomLocalised};
use crate::structure::run::anilist::character::CharacterDataSearch;
use crate::structure::run::anilist::random::{
    Media, MediaFormat, MediaType, RandomPageMedia, RandomPageMediaVariables,
};

/// Executes the command to fetch and display a random anime or manga based on the type specified in the command interaction.
///
/// This function retrieves the type of media (anime or manga) from the command interaction and fetches a random media of that type.
/// It first checks the cache to see if there is a cached response for the specified type. If there is, and the cache was updated within the last 24 hours, it uses the cached response.
/// If there is no cached response or the cache is outdated, it updates the cache by fetching the media data from the AniList API.
/// It then sends an embed containing the media data as a response to the command interaction.
///
/// # Arguments
///
/// * `ctx` - The context in which this command is being executed.
/// * `command_interaction` - The interaction that triggered this command.
///
/// # Returns
///
/// A `Result` that is `Ok` if the command executed successfully, or `Err` if an error occurred.
pub async fn run(ctx: &Context, command_interaction: &CommandInteraction) -> Result<(), AppError> {
    // Retrieve the guild ID from the command interaction
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };

    // Load the localized random strings
    let random_localised = load_localization_random(guild_id).await?;

    // Retrieve the type of media (anime or manga) from the command interaction
    let map = get_option_map_string_subcommand(command_interaction);
    let random_type = map.get(&String::from("type")).ok_or(AppError::new(
        String::from("There is no option"),
        ErrorType::Option,
        ErrorResponseType::Message,
    ))?;

    // Create a deferred response to the command interaction
    let builder_message = Defer(CreateInteractionResponseMessage::new());

    // Send the deferred response
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

    let random_stats = update_random_stats().await?;
    let last_page = if random_type.as_str() == "anime" {
        random_stats.anime_last_page
    } else if random_type.as_str() == "manga" {
        random_stats.manga_last_page
    } else {
        0
    };
    embed(
        last_page,
        random_type.to_string(),
        ctx,
        command_interaction,
        random_localised,
    )
    .await
}

/// Generates and sends an embed containing information about a random anime or manga.
///
/// This function generates a random number between 1 and the last page number of the media list, and fetches a media item from the corresponding page.
/// If the specified media type is "manga", it fetches a manga page; if the media type is "anime", it fetches an anime page.
/// It then constructs a URL for the media item and sends a follow-up message containing an embed with the media information.
///
/// # Arguments
///
/// * `last_page` - The last page number of the media list.
/// * `random_type` - The type of media to fetch ("anime" or "manga").
/// * `ctx` - The context in which this command is being executed.
/// * `command_interaction` - The interaction that triggered this command.
/// * `random_localised` - The localized strings for the random command.
///
/// # Returns
///
/// A `Result` that is `Ok` if the command executed successfully, or `Err` if an error occurred.
async fn embed(
    last_page: i32,
    random_type: String,
    ctx: &Context,
    command_interaction: &CommandInteraction,
    random_localised: RandomLocalised,
) -> Result<(), AppError> {
    let number = thread_rng().gen_range(1..=last_page);
    let mut var = RandomPageMediaVariables {
        media_type: None,
        page: Some(number),
    };
    let mut url = String::new();
    if random_type == "manga" {
        var.media_type = Some(MediaType::Manga)
    } else {
        var.media_type = Some(MediaType::Anime);
    }

    let operation = RandomPageMedia::build(var);
    let data: GraphQlResponse<RandomPageMedia> = match make_request_anilist(operation, false).await
    {
        Ok(data) => match data.json::<GraphQlResponse<RandomPageMedia>>().await {
            Ok(data) => data,
            Err(e) => {
                tracing::error!(?e);
                return Err(AppError {
                    message: format!(
                        "Error retrieving random {} with page: {} \n {}",
                        random_type, number, e
                    ),
                    error_type: ErrorType::WebRequest,
                    error_response_type: ErrorResponseType::Message,
                });
            }
        },
        Err(e) => {
            tracing::error!(?e);
            return Err(AppError {
                message: format!(
                    "Error retrieving random {} with page: {} \n {}",
                    random_type, number, e
                ),
                error_type: ErrorType::WebRequest,
                error_response_type: ErrorResponseType::Message,
            });
        }
    };
    let data = data.data.unwrap();
    let inside_media = data.page.unwrap().media.unwrap()[0].clone().unwrap();
    let id = inside_media.id;
    if random_type == "manga" {
        url = format!("https://anilist.co/manga/{}", id);
    } else {
        url = format!("https://anilist.co/anime/{}", id);
    }
    follow_up_message(
        ctx,
        command_interaction,
        inside_media,
        url,
        random_localised,
    )
    .await?;

    Ok(())
}

/// Sends a follow-up message containing an embed with information about a random anime or manga.
///
/// This function constructs an embed containing information about a random anime or manga, including the title, format, genres, tags, and description.
/// The description is converted from AniList flavored markdown to Discord flavored markdown, and trimmed if it exceeds the maximum length of 4096 characters.
/// The embed also includes a URL to the media item on AniList.
/// The function then sends a follow-up message to the command interaction containing the embed.
///
/// # Arguments
///
/// * `ctx` - The context in which this command is being executed.
/// * `command_interaction` - The interaction that triggered this command.
/// * `data` - The data for the media item to include in the embed.
/// * `url` - The URL to the media item on AniList.
/// * `random_localised` - The localized strings for the random command.
///
/// # Returns
///
/// A `Result` that is `Ok` if the command executed successfully, or `Err` if an error occurred.
async fn follow_up_message(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    media: Media,
    url: String,
    random_localised: RandomLocalised,
) -> Result<(), AppError> {
    let format = media.format.clone().unwrap();
    let genres = media
        .genres
        .unwrap()
        .into_iter()
        .map(|genre| genre.unwrap().clone())
        .collect::<Vec<String>>()
        .join("/");
    let tags = media
        .tags
        .unwrap()
        .into_iter()
        .map(|tag| tag.unwrap().name.clone())
        .collect::<Vec<String>>()
        .join("/");
    let mut desc = media.description.unwrap();
    desc = convert_anilist_flavored_to_discord_flavored_markdown(desc);
    let length_diff = 4096 - desc.len() as i32;
    if length_diff <= 0 {
        desc = trim(desc.clone(), length_diff);
    }
    let title = media.title.clone().unwrap();
    let rj = title.native.unwrap_or_default();
    let user_pref = title.user_preferred.unwrap_or_default();
    let title = format!("{}/{}", user_pref, rj);

    let full_desc = random_localised
        .desc
        .replace("$format$", format.to_string().as_str())
        .replace("$tags$", tags.as_str())
        .replace("$genres$", genres.as_str())
        .replace("$desc$", desc.as_str());

    let builder_embed = get_default_embed(None)
        .title(title)
        .description(full_desc)
        .url(url);

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
