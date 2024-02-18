use std::io::{Cursor, Read};

use base64::engine::general_purpose::STANDARD;
use base64::read::DecoderReader;
use base64::Engine as _;
use image::imageops::FilterType;
use image::{guess_format, GenericImageView, ImageFormat};
use reqwest::get;
use serde_json::json;
use serenity::all::CreateInteractionResponse::Defer;
use serenity::all::{
    ChannelId, CommandInteraction, Context, CreateAttachment, CreateEmbed,
    CreateInteractionResponseFollowup, CreateInteractionResponseMessage, EditWebhook, Timestamp,
};
use tracing::{error, trace};

use crate::anilist_struct::run::minimal_anime::{MinimalAnimeWrapper, Title};
use crate::command_run::get_option::get_option_map_string;
use crate::common::trimer::trim_webhook;
use crate::constant::COLOR;
use crate::database::dispatcher::data_dispatch::{get_one_activity, set_data_activity};
use crate::database_struct::server_activity_struct::ServerActivityFull;
use crate::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use crate::lang_struct::anilist::add_activity::load_localization_add_activity;

pub async fn run(ctx: &Context, command_interaction: &CommandInteraction) -> Result<(), AppError> {
    let map = get_option_map_string(command_interaction);
    let delay = map
        .get(&String::from("delay"))
        .unwrap_or(&String::from("0"))
        .parse()
        .unwrap_or(0);
    let anime = map
        .get(&String::from("anime_name"))
        .cloned()
        .unwrap_or(String::new());

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
    let add_activity_localised = load_localization_add_activity(guild_id.clone()).await?;

    let data = if anime.parse::<i32>().is_ok() {
        MinimalAnimeWrapper::new_minimal_anime_by_id(anime.parse().unwrap()).await?
    } else {
        MinimalAnimeWrapper::new_minimal_anime_by_search(anime.to_string()).await?
    };
    let media = data.data.media.clone();
    let anime_id = media.id;
    let title = data.data.media.title.ok_or(AppError::new(
        String::from("There is no option in the title."),
        ErrorType::Option,
        ErrorResponseType::Message,
    ))?;
    let mut anime_name = get_name(title);
    let channel_id = command_interaction.channel_id;

    if check_if_activity_exist(anime_id, guild_id.clone()).await {
        let builder_embed = CreateEmbed::new()
            .timestamp(Timestamp::now())
            .color(COLOR)
            .title(&add_activity_localised.fail)
            .url(format!("https://anilist.co/anime/{}", media.id))
            .description(
                add_activity_localised
                    .fail_desc
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
    } else {
        if anime_name.len() >= 50 {
            anime_name = trim_webhook(anime_name.clone(), 50 - anime_name.len() as i32)
        }

        let bytes = get(media.cover_image.unwrap().extra_large.
            unwrap_or(
                "https://imgs.search.brave.com/ CYnhSvdQcm9aZe3wG84YY0B19zT2wlAuAkiAGu0mcLc/rs:fit:640:400:1/g:ce/aHR0cDovL3d3dy5m/cmVtb250Z3VyZHdh/cmEub3JnL3dwLWNv/bnRlbnQvdXBsb2Fk/cy8yMDIwLzA2L25v/LWltYWdlLWljb24t/Mi5wbmc"
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
        let base64 = STANDARD.encode(buf.into_inner());
        let image = format!("data:image/jpeg;base64,{}", base64);

        let next_airing = match media.next_airing_episode.clone() {
            Some(na) => na,
            None => {
                return Err(AppError::new(
                    String::from("There is no next airing episode."),
                    ErrorType::Option,
                    ErrorResponseType::Message,
                ))
            }
        };

        let webhook =
            get_webhook(ctx, channel_id, image, base64.clone(), anime_name.clone()).await?;

        set_data_activity(ServerActivityFull {
            anime_id,
            timestamp: next_airing.airing_at.unwrap_or(0),
            guild_id,
            webhook,
            episode: next_airing.episode.unwrap_or(0),
            name: anime_name.clone(),
            delays: delay,
            image: base64,
        })
        .await?;

        let builder_embed = CreateEmbed::new()
            .timestamp(Timestamp::now())
            .color(COLOR)
            .title(&add_activity_localised.success)
            .url(format!("https://anilist.co/anime/{}", media.id))
            .description(
                add_activity_localised
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
}

async fn check_if_activity_exist(anime_id: i32, server_id: String) -> bool {
    let row: (Option<String>, Option<String>, Option<String>) =
        get_one_activity(anime_id, server_id)
            .await
            .unwrap_or((None, None, None));
    !(row.0.is_none() && row.1.is_none() && row.2.is_none())
}

pub fn get_name(title: Title) -> String {
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

async fn get_webhook(
    ctx: &Context,
    channel_id: ChannelId,
    image: String,
    base64: String,
    anime_name: String,
) -> Result<String, AppError> {
    let map = json!({
        "avatar": image,
        "name": anime_name
    });

    let bot_id = match ctx.http.get_current_application_info().await {
        Ok(id) => id.id.to_string(),
        Err(e) => {
            error!("{}", e);
            String::new()
        }
    };

    trace!(bot_id);
    let mut webhook_return = String::new();

    let webhooks = match ctx.http.get_channel_webhooks(channel_id).await {
        Ok(vec) => vec,
        Err(_) => {
            let webhook = ctx
                .http
                .create_webhook(channel_id, &map, None)
                .await
                .map_err(|e| {
                    AppError::new(
                        format!("Error when creating the webhook. {}", e),
                        ErrorType::WebRequest,
                        ErrorResponseType::Followup,
                    )
                })?;
            webhook_return = webhook.url().map_err(|e| {
                AppError::new(
                    format!("Error when getting the webhook url. {}", e),
                    ErrorType::WebRequest,
                    ErrorResponseType::Followup,
                )
            })?;

            return Ok(webhook_return);
        }
    };
    if webhooks.is_empty() {
        let webhook = ctx
            .http
            .create_webhook(channel_id, &map, None)
            .await
            .map_err(|e| {
                AppError::new(
                    format!("Error when creating the webhook. {}", e),
                    ErrorType::WebRequest,
                    ErrorResponseType::Followup,
                )
            })?;
        webhook_return = webhook.url().map_err(|e| {
            AppError::new(
                format!("Error when getting the webhook url. {}", e),
                ErrorType::WebRequest,
                ErrorResponseType::Followup,
            )
        })?;

        return Ok(webhook_return);
    }
    for webhook in webhooks {
        trace!("{:#?}", webhook);
        let webhook_user_id = webhook.user.clone().unwrap().id.to_string();
        trace!(webhook_user_id);
        if webhook_user_id == bot_id {
            trace!("Getting webhook");
            webhook_return = webhook.url().map_err(|e| {
                AppError::new(
                    format!("Error when getting the webhook url. {}", e),
                    ErrorType::WebRequest,
                    ErrorResponseType::Followup,
                )
            })?;
        } else {
            trace!(webhook_return);
            let is_ok = webhook_return == String::new();
            trace!(is_ok);
            if is_ok {
                trace!("Creating webhook");
                let webhook = ctx
                    .http
                    .create_webhook(channel_id, &map, None)
                    .await
                    .map_err(|e| {
                        AppError::new(
                            format!("Error when creating the webhook url. {}", e),
                            ErrorType::WebRequest,
                            ErrorResponseType::Followup,
                        )
                    })?;
                webhook_return = webhook.url().map_err(|e| {
                    AppError::new(
                        format!("Error when getting the webhook url. {}", e),
                        ErrorType::WebRequest,
                        ErrorResponseType::Followup,
                    )
                })?;
            }
        }
    }
    trace!("Done");
    trace!(webhook_return);
    let cursor = Cursor::new(base64);
    let mut decoder = DecoderReader::new(cursor, &STANDARD);

    // Read the decoded bytes into a Vec
    let mut decoded_bytes = Vec::new();
    decoder
        .read_to_end(&mut decoded_bytes)
        .expect("Failed to decode base64");
    let mut webhook = ctx
        .http
        .get_webhook_from_url(webhook_return.as_str())
        .await
        .unwrap();
    let attachement = CreateAttachment::bytes(decoded_bytes, "avatar");
    let edit_webhook = EditWebhook::new().name(anime_name).avatar(&attachement);
    webhook.edit(&ctx.http, edit_webhook).await.unwrap();

    Ok(webhook_return)
}
