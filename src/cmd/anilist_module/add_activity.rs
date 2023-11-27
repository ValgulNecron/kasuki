use std::io::Cursor;

use base64::{engine::general_purpose, Engine as _};
use image::imageops::FilterType;
use image::{guess_format, GenericImageView, ImageFormat};
use reqwest::get;
use serde_json::json;
use serenity::builder::CreateApplicationCommand;
use serenity::client::Context;
use serenity::model::application::command::CommandOptionType;
use serenity::model::prelude::application_command::{
    ApplicationCommandInteraction, CommandDataOption, CommandDataOptionValue,
};
use serenity::model::{Permissions, Timestamp};

use crate::constant::COLOR;
use crate::error_enum::AppError::{LangageGuildIdError, NoAnimeError};
use crate::error_enum::{AppError, COMMAND_SENDING_ERROR};
use crate::function::general::differed_response::differed_response;
use crate::function::general::trim::trim_webhook;
use crate::function::sqls::general::data::set_data_activity;
use crate::function::sqls::sqlite::pool::get_sqlite_pool;
use crate::structure::anilist::struct_minimal_anime::MinimalAnimeWrapper;
use crate::structure::embed::anilist::struct_lang_add_activity::AddActivityLocalisedText;
use crate::structure::register::anilist::struct_add_activity_register::RegisterLocalisedAddActivity;

pub async fn run(
    options: &[CommandDataOption],
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) -> Result<(), AppError> {
    differed_response(ctx, command).await;

    let mut value = "".to_string();
    let mut delays = 0;
    for option in options {
        if option.name == "anime_name" {
            let resolved = option.resolved.as_ref().unwrap();
            if let CommandDataOptionValue::String(value_option) = resolved {
                value = value_option.clone()
            } else {
                return Err(NoAnimeError(String::from("No anime was specified.")));
            }
        }
        if option.name == "delays" {
            let resolved = option.resolved.as_ref().unwrap();
            if let CommandDataOptionValue::Integer(delays_option) = resolved {
                delays = *delays_option
            } else {
                delays = 0;
            }
        }
    }

    let guild_id = command
        .guild_id
        .ok_or(LangageGuildIdError(String::from(
            "Guild id for langage not found.",
        )))?
        .0
        .to_string();

    let localised_text =
        AddActivityLocalisedText::get_add_activity_localised(guild_id.clone()).await?;
    let data = if value.parse::<i32>().is_ok() {
        match MinimalAnimeWrapper::new_minimal_anime_by_id(
            localised_text.clone(),
            value.parse().unwrap(),
        )
        .await
        {
            Ok(minimal_anime) => minimal_anime,
            Err(_) => return Err(NoAnimeError(String::from("No anime was specified."))),
        }
    } else {
        match MinimalAnimeWrapper::new_minimal_anime_by_search(
            localised_text.clone(),
            value.to_string(),
        )
        .await
        {
            Ok(minimal_anime) => minimal_anime,
            Err(_) => return Err(NoAnimeError(String::from("No anime was specified."))),
        }
    };
    let anime_id = data.get_id();

    let mut anime_name = data.get_name();
    let channel_id = command.channel_id.0;
    return if check_if_activity_exist(anime_id, guild_id.clone()).await {
        command
            .create_followup_message(&ctx.http, |f| {
                f.embed(|m| {
                    m.title(&localised_text.title1)
                        .url(format!("https://anilist.co/anime/{}", data.get_id()))
                        .timestamp(Timestamp::now())
                        .color(COLOR)
                        .description(format!(
                            "{} {}",
                            &localised_text.already_added,
                            data.get_name()
                        ))
                })
            })
            .await
            .map_err(|_| COMMAND_SENDING_ERROR.clone())?;
        Ok(())
    } else {
        if anime_name.len() >= 50 {
            anime_name = trim_webhook(anime_name.clone(), 50 - anime_name.len() as i32)
        }
        let bytes = get(data.get_image()).await.unwrap().bytes().await.unwrap();
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
        let webhook = ctx
            .http
            .create_webhook(channel_id, &map, None)
            .await
            .unwrap()
            .url()
            .unwrap();

        set_data_activity(
            anime_id,
            data.get_timestamp(),
            guild_id,
            webhook,
            data.get_episode(),
            data.get_name(),
            delays,
        )
        .await;

        command
            .create_followup_message(&ctx.http, |f| {
                f.embed(|m| {
                    m.title(&localised_text.title2)
                        .url(format!("https://anilist.co/anime/{}", data.get_id()))
                        .timestamp(Timestamp::now())
                        .color(COLOR)
                        .description(format!("{} {}", &localised_text.adding, data.get_name()))
                })
            })
            .await
            .map_err(|_| COMMAND_SENDING_ERROR.clone())?;
        Ok(())
    };
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    let activities = RegisterLocalisedAddActivity::get_add_activity_register_localised().unwrap();
    let command = command
        .name("add_activity")
        .description("Add an anime activity")
        .create_option(|option| {
            let option = option
                .name("anime_name")
                .description("Name of the anime you want to add as an activity")
                .kind(CommandOptionType::String)
                .required(true)
                .set_autocomplete(true);
            for activity in activities.values() {
                option
                    .name_localized(&activity.code, &activity.option1)
                    .description_localized(&activity.code, &activity.option1_desc);
            }
            option
        })
        .create_option(|option| {
            let option = option
                .name("delays")
                .description("A delays in second")
                .kind(CommandOptionType::Integer)
                .required(false);
            for activity in activities.values() {
                option
                    .name_localized(&activity.code, &activity.option2)
                    .description_localized(&activity.code, &activity.option2_desc);
            }
            option
        })
        .default_member_permissions(Permissions::ADMINISTRATOR);
    for activity in activities.values() {
        command
            .name_localized(&activity.code, &activity.name)
            .description_localized(&activity.code, &activity.desc);
    }
    command
}

pub async fn check_if_activity_exist(anime_id: i32, server_id: String) -> bool {
    let database_url = "./data.db";
    let pool = get_sqlite_pool(database_url).await;
    let row: (Option<String>, Option<String>, Option<String>, Option<String>) = sqlx::query_as(
        "SELECT anime_id, timestamp, server_id, webhook FROM activity_data WHERE anime_id = ? AND server_id = ?",
    )
        .bind(anime_id)
        .bind(server_id)
        .fetch_one(&pool)
        .await
        .unwrap_or((None, None, None, None));
    !(row.0.is_none() && row.1.is_none() && row.2.is_none() && row.3.is_none())
}
