use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use crate::cmd::general_module::lang_struct::RandomLocalisedText;
use crate::constant::COLOR;
use crate::function::error_management::no_lang_error::{
    error_cant_read_langage_file, error_langage_file_not_found, error_no_langage_guild_id,
    error_parsing_langage_json,
};
use crate::function::general::differed_response::differed_response;
use crate::function::general::get_guild_langage::get_guild_langage;
use crate::function::sql::sqlite::pool::get_sqlite_pool;
use crate::structure::anilist::random::struct_random::PageWrapper;
use crate::structure::anilist::random::struct_site_statistic_anime::SiteStatisticsAnimeWrapper;
use crate::structure::anilist::random::struct_site_statistic_manga::SiteStatisticsMangaWrapper;
use chrono::Utc;
use rand::prelude::*;
use serenity::builder::CreateApplicationCommand;
use serenity::client::Context;
use serenity::model::application::interaction::application_command::CommandDataOptionValue;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::interaction::application_command::{
    ApplicationCommandInteraction, CommandDataOption,
};
use serenity::model::Timestamp;
use sqlx::{Pool, Sqlite};

pub async fn run(
    options: &[CommandDataOption],
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) {
    let database_url = "./cache.db";
    let pool = get_sqlite_pool(database_url).await;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS cache_stats (
            key TEXT PRIMARY KEY,
            response TEXT NOT NULL,
            last_updated INTEGER NOT NULL,
            last_page INTEGER NOT NULL
        )",
    )
    .execute(&pool)
    .await
    .unwrap();

    let option = options
        .get(0)
        .expect("Expected username option")
        .resolved
        .as_ref()
        .expect("Expected username object");

    if let CommandDataOptionValue::String(random_type) = option {
        differed_response(ctx, command).await;

        let row: (Option<String>, Option<i64>, Option<i64>) = sqlx::query_as(
            "SELECT response, last_updated, last_page FROM cache_stats WHERE key = ?",
        )
        .bind(random_type)
        .fetch_one(&pool)
        .await
        .unwrap_or((None, None, None));

        let (response, last_updated, last_page): (Option<String>, Option<i64>, Option<i64>) = row;

        let page_number = last_page.unwrap_or(1567); // This is as today date the last page, i will update it sometime.

        let previous_page = page_number - 1;
        let cached_response = response.unwrap_or("Nothing".to_string());

        if let Some(updated) = last_updated {
            let duration_since_updated = Utc::now().timestamp() - updated;
            if duration_since_updated < 24 * 60 * 60 {
                embed(page_number, random_type.to_string(), ctx, command).await;
            } else {
                update_cache(
                    page_number,
                    random_type,
                    ctx,
                    command,
                    previous_page,
                    cached_response,
                    pool,
                )
                .await
            }
        } else {
            update_cache(
                page_number,
                random_type,
                ctx,
                command,
                previous_page,
                cached_response,
                pool,
            )
            .await
        }
    }
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("random")
        .description("Get a random manga or anime")
        .create_option(|option| {
            option
                .name("type")
                .description("Type of the media you want manga or anime. manga include ln atm.")
                .kind(CommandOptionType::String)
                .required(true)
                .add_string_choice("manga", "manga")
                .add_string_choice("anime", "anime")
        })
}

pub async fn embed(
    last_page: i64,
    random_type: String,
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) {
    let number = thread_rng().gen_range(1..=last_page);
    if random_type == "manga" {
        let data = PageWrapper::new_manga_page(number).await;

        let url = data.get_manga_url();

        follow_up_message(ctx, command, data, url).await
    } else if random_type == "anime" {
        let data = PageWrapper::new_anime_page(number).await;

        let url = data.get_anime_url();

        follow_up_message(ctx, command, data, url).await
    } else {
        let mut file = match File::open("./lang_file/embed/anilist/random.json") {
            Ok(file) => file,
            Err(_) => {
                error_langage_file_not_found(ctx, command).await;
                return;
            }
        };
        let mut json = String::new();
        match file.read_to_string(&mut json) {
            Ok(_) => {}
            Err(_) => error_cant_read_langage_file(ctx, command).await,
        }

        let json_data: HashMap<String, RandomLocalisedText> = match serde_json::from_str(&json) {
            Ok(data) => data,
            Err(_) => {
                error_parsing_langage_json(ctx, command).await;
                return;
            }
        };

        let guild_id = match command.guild_id {
            Some(id) => id.0.to_string(),
            None => {
                error_no_langage_guild_id(ctx, command).await;
                return;
            }
        };
        let lang_choice = get_guild_langage(guild_id).await;

        if let Some(localised_text) = json_data.get(lang_choice.as_str()) {
            if let Err(why) = command
                .create_followup_message(&ctx.http, |f| {
                    f.embed(|m| {
                        m.title(&localised_text.error_title)
                            .description(&localised_text.error_message)
                            .timestamp(Timestamp::now())
                            .color(COLOR)
                    })
                })
                .await
            {
                println!("Cannot respond to slash command: {}", why);
            }
        }
    }
}

pub async fn follow_up_message(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    data: PageWrapper,
    url: String,
) {
    let mut file =
        File::open("../../../lang_file/embed/anilist/random.json").expect("Failed to open file");
    let mut json = String::new();
    file.read_to_string(&mut json).expect("Failed to read file");

    let title_user = data.get_user_pref_title();
    let title = data.get_native_title();
    let genre = data.get_genre();
    let tag = data.get_tags();
    let format = data.get_format();
    let description = data.get_description();
    let cover_image = data.get_cover_image();

    let json_data: HashMap<String, RandomLocalisedText> =
        serde_json::from_str(&json).expect("Failed to parse JSON");

    let guild_id = command.guild_id.unwrap().0.to_string().clone();
    let lang_choice = get_guild_langage(guild_id).await;

    if let Some(localised_text) = json_data.get(lang_choice.as_str()) {
        if let Err(why) = command
            .create_followup_message(&ctx.http, |f| {
                f.embed(|m| {
                    m.title(format!("{}/{}", title_user, title))
                        .description(format!(
                            "{}{}{}{}{}{}{}{}",
                            &localised_text.genre,
                            genre,
                            &localised_text.tag,
                            tag,
                            &localised_text.format,
                            format,
                            &localised_text.desc,
                            description
                        ))
                        .timestamp(Timestamp::now())
                        .color(COLOR)
                        .thumbnail(cover_image)
                        .url(url)
                })
            })
            .await
        {
            println!("{}: {}", localised_text.error_slash_command, why);
        }
    }
}

pub async fn update_cache(
    mut page_number: i64,
    random_type: &String,
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    mut previous_page: i64,
    mut cached_response: String,
    pool: Pool<Sqlite>,
) {
    let now = Utc::now().timestamp();

    if random_type.as_str() == "manga" {
        loop {
            let (data, res) = SiteStatisticsMangaWrapper::new_manga(page_number).await;
            let has_next_page = data.has_next_page();

            if !has_next_page {
                break;
            }
            cached_response = res.to_string();
            previous_page = page_number;

            page_number += 1;
        }
    } else if random_type.as_str() == "anime" {
        loop {
            let (data, res) = SiteStatisticsAnimeWrapper::new_anime(page_number).await;
            let has_next_page = data.has_next_page();

            if !has_next_page {
                break;
            }
            cached_response = res.to_string();
            previous_page = page_number;

            page_number += 1;
        }
    }

    sqlx::query("INSERT OR REPLACE INTO cache_stats (key, response, last_updated, last_page) VALUES (?, ?, ?, ?)")
        .bind(random_type)
        .bind(&cached_response)
        .bind(now)
        .bind(previous_page)
        .execute(&pool)
        .await.unwrap();
    embed(previous_page, random_type.to_string(), ctx, command).await;
}
