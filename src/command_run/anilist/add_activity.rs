use crate::anilist_struct::run::minimal_anime::{MinimalAnimeWrapper, Title};
use crate::common::trimer::trim_webhook;
use crate::constant::{
    COLOR, COMMAND_SENDING_ERROR, DIFFERED_COMMAND_SENDING_ERROR, DIFFERED_OPTION_ERROR,
};
use crate::error_enum::AppError;
use crate::error_enum::AppError::{CreatingWebhookDifferedError, DifferedNotAiringError};
use crate::lang_struct::anilist::add_activity::load_localization_add_activity;
use crate::sqls::general::data::{get_one_activity, set_data_activity};
use base64::{engine::general_purpose, Engine as _};
use image::imageops::FilterType;
use image::{guess_format, GenericImageView, ImageFormat};
use reqwest::get;
use serde_json::json;
use serenity::all::CreateInteractionResponse::Defer;
use serenity::all::{
    CommandDataOption, CommandDataOptionValue, CommandInteraction, Context, CreateEmbed,
    CreateInteractionResponseFollowup, CreateInteractionResponseMessage, Timestamp,
};
use std::io::Cursor;

pub async fn run(
    options: &[CommandDataOption],
    ctx: &Context,
    command: &CommandInteraction,
) -> Result<(), AppError> {
    let mut delay = 0;
    let mut anime = String::new();
    for option in options {
        if option.name == "delay" {
            let resolved = &option.value;
            if let CommandDataOptionValue::Integer(delay_option) = resolved {
                delay = delay_option.clone()
            } else {
                delay = 0;
            }
        }
        if option.name == "anime_name" {
            let resolved = &option.value;
            if let CommandDataOptionValue::String(anime_option) = resolved {
                anime = anime_option.clone()
            } else {
                anime = String::new()
            }
        }
    }

    let guild_id = match command.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };

    let builder_message = Defer(CreateInteractionResponseMessage::new());

    command
        .create_response(&ctx.http, builder_message)
        .await
        .map_err(|_| COMMAND_SENDING_ERROR.clone())?;

    let add_activity_localised = load_localization_add_activity(guild_id.clone()).await?;

    let data = if anime.parse::<i32>().is_ok() {
        MinimalAnimeWrapper::new_minimal_anime_by_id(anime.parse().unwrap()).await?
    } else {
        MinimalAnimeWrapper::new_minimal_anime_by_search(anime.to_string()).await?
    };
    let media = data.data.media.clone();
    let anime_id = media.id.clone();
    let title = data.data.media.title.ok_or(DIFFERED_OPTION_ERROR.clone())?;
    let mut anime_name = get_name(title);
    let channel_id = command.channel_id;
    if check_if_activity_exist(anime_id, guild_id.clone()).await {
        let builder_embed = CreateEmbed::new()
            .timestamp(Timestamp::now())
            .color(COLOR)
            .title(&add_activity_localised.fail)
            .url(format!("https://anilist.co/anime/{}", media.id))
            .description(
                &add_activity_localised
                    .fail_desc
                    .replace("$anime$", anime_name.as_str()),
            );

        let builder_message = CreateInteractionResponseFollowup::new().embed(builder_embed);

        command
            .create_followup(&ctx.http, builder_message)
            .await
            .map_err(|_| DIFFERED_COMMAND_SENDING_ERROR.clone())?;

        Ok(())
    } else {
        if anime_name.len() >= 50 {
            anime_name = trim_webhook(anime_name.clone(), 50 - anime_name.len() as i32)
        }

        let bytes = get(media.cover_image.unwrap().extra_large.
        unwrap_or(
            "https://imgs.search.brave.com/CYnhSvdQcm9aZe3wG84YY0B19zT2wlAuAkiAGu0mcLc/rs:fit:640:400:1/g:ce/aHR0cDovL3d3dy5m/cmVtb250Z3VyZHdh/cmEub3JnL3dwLWNv/bnRlbnQvdXBsb2Fk/cy8yMDIwLzA2L25v/LWltYWdlLWljb24t/Mi5wbmc"
                .to_string()
        )
).await.unwrap().bytes().await.unwrap();
        let mut img = image::load(Cursor::new(&bytes), guess_format(&bytes).unwrap()).unwrap();
        let (width, height) = img.dimensions();
        let square_size = width.min(height);
        let crop_x = (width - square_size) / 2;
        let crop_y = (height - square_size) / 2;

        let img = img
            .crop(crop_x, crop_y, square_size, square_size)
            .resize_exact(128, 128, FilterType::Nearest);
        let mut buf = Cursor::new(Vec::new());
        img.write_to(&mut buf, ImageFormat::Jpeg)
            .expect("Failed to encode image");
        let base64 = general_purpose::STANDARD.encode(buf.into_inner());
        let image = format!("data:image/jpeg;base64,{}", base64);
        let map = json!({
            "avatar": image,
            "name": anime_name
        });

        let next_airing = match media.next_airing_episode.clone() {
            Some(na) => na,
            None => return Err(DifferedNotAiringError(String::from("Not airing"))),
        };

        let webhook = ctx
            .http
            .create_webhook(channel_id, &map, None)
            .await
            .map_err(|_| {
                CreatingWebhookDifferedError(String::from("Error when creating the webhook."))
            })?
            .url()
            .map_err(|_| {
                CreatingWebhookDifferedError(String::from("Error when getting the webhook url."))
            })?;

        set_data_activity(
            anime_id,
            next_airing.airing_at.unwrap_or(0),
            guild_id,
            webhook,
            next_airing.episode.unwrap_or(0),
            anime_name.clone(),
            delay,
        )
        .await?;

        let builder_embed = CreateEmbed::new()
            .timestamp(Timestamp::now())
            .color(COLOR)
            .title(&add_activity_localised.success)
            .url(format!("https://anilist.co/anime/{}", media.id))
            .description(
                &add_activity_localised
                    .success_desc
                    .replace("$anime$", anime_name.as_str()),
            );

        let builder_message = CreateInteractionResponseFollowup::new().embed(builder_embed);

        command
            .create_followup(&ctx.http, builder_message)
            .await
            .map_err(|_| DIFFERED_COMMAND_SENDING_ERROR.clone())?;

        Ok(())
    }
}

async fn check_if_activity_exist(anime_id: i32, server_id: String) -> bool {
    let row: (
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
    ) = get_one_activity(anime_id, server_id)
        .await
        .unwrap_or((None, None, None, None));
    !(row.0.is_none() && row.1.is_none() && row.2.is_none() && row.3.is_none())
}

fn get_name(title: Title) -> String {
    let en = title.english.clone();
    let rj = title.romaji.clone();
    let en = en.unwrap_or(String::from(""));
    let rj = rj.unwrap_or(String::from(""));
    let mut title = String::new();
    let mut total = 0;
    match en.as_str() {
        "" => {}
        _ => {
            total += 1;
            title.push_str(en.as_str())
        }
    }
    match rj.as_str() {
        "\"\"" => {}
        _ => {
            if total == 1 {
                title.push_str(" / ");
                title.push_str(rj.as_str())
            } else {
                title.push_str(rj.as_str())
            }
        }
    }

    title
}