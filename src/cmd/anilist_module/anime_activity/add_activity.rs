use std::collections::HashMap;
use std::fs::File;
use std::io::{Cursor, Read};

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
use serenity::model::prelude::autocomplete::AutocompleteInteraction;
use serenity::model::{Permissions, Timestamp};
use serenity::utils::Colour;

use crate::cmd::anilist_module::anime_activity::struct_minimal_anime::MinimalAnimeWrapper;
use crate::cmd::anilist_module::struct_autocomplete_media::MediaPageWrapper;
use crate::cmd::general_module::differed_response::differed_response;
use crate::cmd::general_module::get_guild_langage::get_guild_langage;
use crate::cmd::general_module::lang_struct::AddActivityLocalisedText;
use crate::cmd::general_module::pool::get_pool;
use crate::cmd::general_module::trim::trim_100_webhook;

pub async fn run(
    options: &[CommandDataOption],
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) -> String {
    let result_diff = differed_response(ctx, command).await;

    if result_diff != "good".as_ref() {
        return result_diff;
    }
    let database_url = "./data.db";
    let pool = get_pool(database_url).await;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS activity_data (
        anime_id TEXT,
        timestamp TEXT,
        server_id TEXT,
        webhook TEXT,
        episode TEXT,
        name TEXT,
        delays INTEGER DEFAULT 0,
        PRIMARY KEY (anime_id, server_id)
    )",
    )
    .execute(&pool)
    .await
    .unwrap();

    let mut value = "".to_string();
    let mut delays = 0;
    for option in options {
        if option.name == "anime_name" {
            let resolved = option.resolved.as_ref().unwrap();
            if let CommandDataOptionValue::String(value_option) = resolved {
                value = value_option.clone()
            } else {
                return "please specify an anime".to_string();
            }
        }
        if option.name == "delays" {
            let resolved = option.resolved.as_ref().unwrap();
            if let CommandDataOptionValue::Integer(delays_option) = resolved {
                delays = delays_option.clone()
            } else {
                delays = 0;
            }
        }
    }
    let mut file = File::open("lang_file/embed/anilist/add_activity.json").expect("Failed to open file");
    let mut json = String::new();
    file.read_to_string(&mut json).expect("Failed to read file");

    let json_data: HashMap<String, AddActivityLocalisedText> =
        serde_json::from_str(&json).expect("Failed to parse JSON");

    let guild_id = command.guild_id.unwrap().0.to_string().clone();
    let lang_choice = get_guild_langage(guild_id.clone()).await;

    if let Some(localised_text) = json_data.get(lang_choice.as_str()) {
        let data;
        if match value.parse::<i32>() {
            Ok(_) => true,
            Err(_) => false,
        } {
            data = match MinimalAnimeWrapper::new_minimal_anime_by_id(
                localised_text.clone(),
                value.parse().unwrap(),
            )
            .await
            {
                Ok(minimal_anime) => minimal_anime,
                Err(error) => return error,
            }
        } else {
            data = match MinimalAnimeWrapper::new_minimal_anime_by_search(
                localised_text.clone(),
                value.to_string(),
            )
            .await
            {
                Ok(minimal_anime) => minimal_anime,
                Err(error) => return error,
            }
        }
        let anime_id = data.get_id();

        let mut anime_name = data.get_name();
        let channel_id = command.channel_id.0;
        let color = Colour::FABLED_PINK;
        if check_if_activity_exist(anime_id, guild_id.clone()).await {
            if let Err(why) = command
                .create_followup_message(&ctx.http, |f| {
                    f.embed(|m| {
                        m.title(&localised_text.title1)
                            .url(format!("https://anilist.co/anime/{}", data.get_id()))
                            .timestamp(Timestamp::now())
                            .color(color)
                            .description(format!(
                                "{} {}",
                                &localised_text.already_added,
                                data.get_name()
                            ))
                            .color(color)
                    })
                })
                .await
            {
                println!("{}: {}", localised_text.error_slash_command, why);
            }
            return "good".to_string();
        } else {
            if anime_name.len() >= 50 {
                anime_name = trim_100_webhook(anime_name.clone(), 50 - anime_name.len() as i32)
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
            let base64 = general_purpose::STANDARD.encode(&buf.into_inner());
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
            sqlx::query(
        "INSERT OR REPLACE INTO activity_data (anime_id, timestamp, server_id, webhook, delays) VALUES (?, ?, ?, ?, ?)",
                )
                    .bind(anime_id)
                    .bind(data.get_timestamp())
                    .bind(guild_id)
                    .bind(webhook)
                    .bind(data.get_episode())
                    .bind(data.get_name())
                                .bind(delays)
                    .execute(&pool)
                    .await
                    .unwrap();
            if let Err(why) = command
                .create_followup_message(&ctx.http, |f| {
                    f.embed(|m| {
                        m.title(&localised_text.title2)
                            .url(format!("https://anilist.co/anime/{}", data.get_id()))
                            .timestamp(Timestamp::now())
                            .color(color)
                            .description(format!("{} {}", &localised_text.adding, data.get_name()))
                            .color(color)
                    })
                })
                .await
            {
                println!("{}: {}", localised_text.error_slash_command, why);
            }
            return "good".to_string();
        }
    }
    "good".to_string()
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("add_activity")
        .description("Add an anime activity")
        .create_option(|option| {
            option
                .name("anime_name")
                .description("Name of the anime you want to add as an activity")
                .kind(CommandOptionType::String)
                .required(true)
                .set_autocomplete(true)
        })
        .create_option(|option| {
            option
                .name("delays")
                .description("A delays in second")
                .kind(CommandOptionType::Integer)
                .required(false)
        })
        .default_member_permissions(Permissions::ADMINISTRATOR)
}

pub async fn autocomplete(ctx: Context, command: AutocompleteInteraction) {
    let search = &command.data.options.first().unwrap().value;
    if let Some(search) = search {
        let data = MediaPageWrapper::new_autocomplete_anime(search, 8, "ANIME").await;
        let choices = data.get_choices();
        // doesn't matter if it errors
        _ = command
            .create_autocomplete_response(ctx.http, |response| {
                response.set_choices(choices.clone())
            })
            .await;
    }
}

pub async fn check_if_activity_exist(anime_id: i32, server_id: String) -> bool {
    let database_url = "./data.db";
    let pool = get_pool(database_url).await;
    let row: (Option<String>, Option<String>, Option<String>, Option<String>) = sqlx::query_as(
        "SELECT anime_id, timestamp, server_id, webhook FROM activity_data WHERE anime_id = ? AND server_id = ?",
    )
        .bind(anime_id)
        .bind(server_id)
        .fetch_one(&pool)
        .await
        .unwrap_or((None, None, None, None));
    let is_row_none = row.0.is_none() && row.1.is_none() && row.2.is_none() && row.3.is_none();

    if is_row_none {
        false
    } else {
        true
    }
}
