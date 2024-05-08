use chrono::Utc;
use rand::{thread_rng, Rng};
use serenity::all::CreateInteractionResponse::Defer;
use serenity::all::{
    CommandInteraction, Context, CreateInteractionResponseFollowup,
    CreateInteractionResponseMessage,
};

use crate::anilist_struct::run::random::PageWrapper;
use crate::anilist_struct::run::site_statistic_anime::SiteStatisticsAnimeWrapper;
use crate::anilist_struct::run::site_statistic_manga::SiteStatisticsMangaWrapper;
use crate::common::anilist_to_discord_markdown::convert_anilist_flavored_to_discord_flavored_markdown;
use crate::common::default_embed::get_default_embed;
use crate::common::get_option::subcommand::get_option_map_string_subcommand;
use crate::common::trimer::trim;
use crate::cache::manage::cache_dispatch::{
    get_database_random_cache, set_database_random_cache,
};
use crate::cache::cache_struct::cache_stats::CacheStats;
use crate::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use crate::lang_struct::anilist_user::random::{load_localization_random, RandomLocalised};

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

    // Retrieve the cached response for the specified type
    let row: Option<CacheStats> = get_database_random_cache(random_type).await?;
    let (cached_response, last_updated, page_number) = match row {
        Some(row) => (row.response, row.last_updated, row.last_page),
        None => (String::new(), 0, 1628),
    };
    let previous_page = page_number - 1;

    // If the cache was updated within the last 24 hours, use the cached response
    if last_updated != 0 {
        let duration_since_updated = Utc::now().timestamp() - last_updated;
        if duration_since_updated < 24 * 60 * 60 {
            return embed(
                page_number,
                random_type.to_string(),
                ctx,
                command_interaction,
                random_localised,
            )
            .await;
        }
    }

    // If there is no cached response or the cache is outdated, update the cache by fetching the media data from the AniList API
    update_cache(
        page_number,
        random_type,
        ctx,
        command_interaction,
        previous_page,
        cached_response,
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
    last_page: i64,
    random_type: String,
    ctx: &Context,
    command_interaction: &CommandInteraction,
    random_localised: RandomLocalised,
) -> Result<(), AppError> {
    let number = thread_rng().gen_range(1..=last_page);
    if random_type == "manga" {
        let data = PageWrapper::new_manga_page(number).await?;
        let url = format!(
            "https://anilist.co/manga/{}",
            data.data.page.media.clone()[0].id
        );
        follow_up_message(ctx, command_interaction, data, url, random_localised).await
    } else if random_type == "anime" {
        let data = PageWrapper::new_anime_page(number).await?;
        let url = format!(
            "https://anilist.co/anime/{}",
            data.data.page.media.clone()[0].id
        );
        follow_up_message(ctx, command_interaction, data, url, random_localised).await
    } else {
        Ok(())
    }
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
    data: PageWrapper,
    url: String,
    random_localised: RandomLocalised,
) -> Result<(), AppError> {
    let media = data.data.page.media.clone()[0].clone();
    let format = media.format.clone();
    let genres = media.genres.join("/");
    let tags = media
        .tags
        .into_iter()
        .map(|tag| tag.name.clone())
        .collect::<Vec<String>>()
        .join("/");
    let mut desc = media.description;
    desc = convert_anilist_flavored_to_discord_flavored_markdown(desc);
    let length_diff = 4096 - desc.len() as i32;
    if length_diff <= 0 {
        desc = trim(desc.clone(), length_diff);
    }
    let rj = media.title.native;
    let user_pref = media.title.user_preferred;
    let title = format!("{}/{}", user_pref, rj);

    let full_desc = random_localised
        .desc
        .replace("$format$", format.as_str())
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

/// Updates the cache with data fetched from the AniList API.
///
/// This function fetches data from the AniList API for a specified type of media (anime or manga), and updates the cache with this data.
/// It first determines the current timestamp, which will be used as the last updated time for the cache.
/// Then, it enters a loop where it fetches data from the AniList API for the specified type of media, page by page, until it reaches a page that does not have a next page.
/// The fetched data is converted to a string and stored in the cache.
/// After the loop, the function updates the cache in the database with the new data and the current timestamp.
/// Finally, it sends an embed containing the cached data as a response to the command interaction.
///
/// # Arguments
///
/// * `page_number` - The page number to start fetching data from.
/// * `random_type` - The type of media to fetch data for ("anime" or "manga").
/// * `ctx` - The context in which this command is being executed.
/// * `command_interaction` - The interaction that triggered this command.
/// * `previous_page` - The page number of the last page that was fetched in the previous run of this function.
/// * `cached_response` - The response that was cached in the previous run of this function.
/// * `random_localised` - The localized strings for the random command.
///
/// # Returns
///
/// A `Result` that is `Ok` if the command executed successfully, or `Err` if an error occurred.
pub async fn update_cache(
    mut page_number: i64,
    random_type: &String,
    ctx: &Context,
    command_interaction: &CommandInteraction,
    mut previous_page: i64,
    mut cached_response: String,
    random_localised: RandomLocalised,
) -> Result<(), AppError> {
    let now = Utc::now().timestamp();

    if random_type.as_str() == "manga" {
        loop {
            let (data, res) = SiteStatisticsMangaWrapper::new_manga(page_number).await?;
            let has_next_page = data.has_next_page();

            if !has_next_page {
                break;
            }
            cached_response = res.to_string();
            previous_page = page_number;

            page_number += 1
        }
    } else if random_type.as_str() == "anime" {
        loop {
            let (data, res) = SiteStatisticsAnimeWrapper::new_anime(page_number).await?;
            let has_next_page = data.has_next_page();

            if !has_next_page {
                break;
            }
            cached_response = res.to_string();
            previous_page = page_number;

            page_number += 1
        }
    }

    set_database_random_cache(random_type, cached_response.as_str(), now, previous_page).await?;
    embed(
        previous_page,
        random_type.to_string(),
        ctx,
        command_interaction,
        random_localised,
    )
    .await
}
