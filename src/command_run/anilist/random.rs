use chrono::Utc;
use rand::{thread_rng, Rng};
use serenity::all::CreateInteractionResponse::Defer;
use serenity::all::{
    CommandDataOption, CommandDataOptionValue, CommandInteraction, Context, CreateEmbed,
    CreateInteractionResponseFollowup, CreateInteractionResponseMessage, Timestamp,
};

use crate::anilist_struct::run::random::PageWrapper;
use crate::anilist_struct::run::site_statistic_anime::SiteStatisticsAnimeWrapper;
use crate::anilist_struct::run::site_statistic_manga::SiteStatisticsMangaWrapper;
use crate::common::anilist_to_discord_markdown::convert_anilist_flavored_to_discord_flavored_markdown;
use crate::common::trimer::trim;
use crate::constant::{COLOR, COMMAND_SENDING_ERROR, DIFFERED_COMMAND_SENDING_ERROR, OPTION_ERROR};
use crate::database::dispatcher::cache_dispatch::{
    get_database_random_cache, set_database_random_cache,
};
use crate::error_enum::AppError;
use crate::lang_struct::anilist::random::{load_localization_random, RandomLocalised};

pub async fn run(
    options: &[CommandDataOption],
    ctx: &Context,
    command_interaction: &CommandInteraction,
) -> Result<(), AppError> {
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };

    let random_localised = load_localization_random(guild_id).await?;

    let builder_message = Defer(CreateInteractionResponseMessage::new());

    command_interaction
        .create_response(&ctx.http, builder_message)
        .await
        .map_err(|_| COMMAND_SENDING_ERROR.clone())?;
    let option = &options.first().ok_or(OPTION_ERROR.clone())?.value;
    if let CommandDataOptionValue::String(random_type) = option {
        let row: (Option<String>, Option<i64>, Option<i64>) =
            get_database_random_cache(random_type).await?;
        let (response, last_updated, last_page): (Option<String>, Option<i64>, Option<i64>) = row;
        let page_number = last_page.unwrap_or(1628); // This is as today date the last page,
                                                     // I will update it sometime.
        let previous_page = page_number - 1;
        let cached_response = response.unwrap_or("Nothing".to_string());
        if let Some(updated) = last_updated {
            let duration_since_updated = Utc::now().timestamp() - updated;
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
    } else {
        Err(AppError::NoCommandOption(String::from(
            "The command contain no option.",
        )))
    }
}

pub async fn embed(
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

pub async fn follow_up_message(
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

    let builder_embed = CreateEmbed::new()
        .timestamp(Timestamp::now())
        .color(COLOR)
        .title(title)
        .description(full_desc)
        .url(url);

    let builder_message = CreateInteractionResponseFollowup::new().embed(builder_embed);

    command_interaction
        .create_followup(&ctx.http, builder_message)
        .await
        .map_err(|_| DIFFERED_COMMAND_SENDING_ERROR.clone())?;
    Ok(())
}

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
