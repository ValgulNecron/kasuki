use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use chrono::Utc;
use rand::prelude::*;
use reqwest::Client;
use serde_json::{json, Value};
use serenity::builder::CreateApplicationCommand;
use serenity::client::Context;
use serenity::model::application::interaction::application_command::CommandDataOptionValue;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::interaction::application_command::{
    ApplicationCommandInteraction, CommandDataOption,
};
use serenity::model::Timestamp;
use serenity::utils::Colour;
use sqlx::{Pool, Sqlite};

use crate::cmd::anilist_module::struct_random::*;
use crate::cmd::general_module::differed_response::differed_response;
use crate::cmd::general_module::get_guild_langage::get_guild_langage;
use crate::cmd::general_module::lang_struct::RandomLocalisedText;
use crate::cmd::general_module::pool::get_pool;
use crate::cmd::general_module::request::make_request;

pub async fn run(
    options: &[CommandDataOption],
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) -> String {
    let database_url = "./cache.db";
    let pool = get_pool(database_url).await;

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

        let page_number = match last_page {
            Some(page) => page,
            None => 1444, // This is as today date the last page, i will update it sometime.
        };

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
    return "good".to_string();
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
    let color = Colour::FABLED_PINK;
    let client = Client::new();
    let number = thread_rng().gen_range(1..=last_page);
    if random_type == "manga" {
        let query = "
                    query($manga_page: Int){
                        Page(page: $manga_page, perPage: 1){
                            media(type: MANGA){
                            id
                            title {
                                native
                                userPreferred
                            }
                            meanScore
                            description
                            tags {
                                name
                            }
                            genres
                            format
                            status
                            coverImage {
                                extraLarge
                            }
                        }
                    }
                }";

        let json = json!({"query": query, "variables": {"manga_page": number}});
        let res = make_request(json).await;

        let api_response: PageData = serde_json::from_str(&res).unwrap();

        let media = &api_response.data.page.media[0];
        let title_user = &media.title.user_preferred;
        let title = &media.title.native;

        let tag = &media.tags;
        let genre = &media.genres;

        let genres_str = genre.join("/");
        let tags_str = tag
            .into_iter()
            .map(|tag| tag.name.clone())
            .collect::<Vec<String>>()
            .join("/");

        let cover_image = &media.cover_image.extra_large;

        let description = &media.description;
        let desc_no_br = description.replace("<br>", "");

        let format = &media.format;

        let url = format!("https://anilist.co/manga/{}", &media.id);
        follow_up_message(
            ctx,
            command,
            genres_str,
            tags_str,
            format,
            desc_no_br,
            title_user,
            title,
            color,
            cover_image,
            url,
        )
        .await;
    } else if random_type == "anime" {
        let query = "
                    query($anime_page: Int){
                        Page(page: $anime_page, perPage: 1){
                            media(type: ANIME){
                            id
                            title {
                                native
                                userPreferred
                            }
                            meanScore
                            description
                            tags {
                                name
                            }
                            genres
                            format
                            status
                            coverImage {
                                extraLarge
                            }
                        }
                    }
                }";

        let json = json!({"query": query, "variables": {"anime_page": number}});
        let res = client
            .post("https://graphql.anilist.co")
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .body(json.to_string())
            .send()
            .await
            .unwrap()
            .text()
            .await;

        let api_response: PageData = serde_json::from_str(&res.unwrap()).unwrap();

        let media = &api_response.data.page.media[0];
        let title_user = &media.title.user_preferred;
        let title = &media.title.native;

        let tag = &media.tags;
        let genre = &media.genres;

        let genres_str = genre.join("/");
        let tags_str = tag
            .into_iter()
            .map(|tag| tag.name.clone())
            .collect::<Vec<String>>()
            .join("/");

        let cover_image = &media.cover_image.extra_large;

        let description = &media.description;
        let desc_no_br = description.replace("<br>", "");

        let format = &media.format;

        let url = format!("https://anilist.co/anime/{}", &media.id);
        follow_up_message(
            ctx,
            command,
            genres_str,
            tags_str,
            format,
            desc_no_br,
            title_user,
            title,
            color,
            cover_image,
            url,
        )
        .await;
    } else {
        let mut file = File::open("lang_file/anilist/random.json").expect("Failed to open file");
        let mut json = String::new();
        file.read_to_string(&mut json).expect("Failed to read file");

        let json_data: HashMap<String, RandomLocalisedText> =
            serde_json::from_str(&json).expect("Failed to parse JSON");

        let guild_id = command.guild_id.unwrap().0.to_string().clone();
        let lang_choice = get_guild_langage(guild_id).await;

        if let Some(localised_text) = json_data.get(lang_choice.as_str()) {
            if let Err(why) = command
                .create_followup_message(&ctx.http, |f| {
                    f.embed(|m| {
                        m.title(&localised_text.error_title)
                            .description(&localised_text.error_message)
                            .timestamp(Timestamp::now())
                            .color(color)
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
    genres_str: String,
    tags_str: String,
    format: &String,
    desc_no_br: String,
    title_user: &String,
    title: &String,
    color: Colour,
    cover_image: &String,
    url: String,
) {
    let mut file = File::open("lang_file/anilist/random.json").expect("Failed to open file");
    let mut json = String::new();
    file.read_to_string(&mut json).expect("Failed to read file");

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
                            genres_str,
                            &localised_text.tag,
                            tags_str,
                            &localised_text.format,
                            format,
                            &localised_text.desc,
                            desc_no_br
                        ))
                        .timestamp(Timestamp::now())
                        .color(color)
                        .thumbnail(cover_image)
                        .url(url)
                })
            })
            .await
        {
            println!("Cannot respond to slash command: {}", why);
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

    loop {
        let client = Client::new();
        let query;
        if random_type.as_str() == "manga" {
            query = json!({
                "query": r#"
                    query($manga_page: Int = 1444){
                        SiteStatistics{
                            manga(perPage: 1, page: $manga_page){
                                pageInfo{
                                    currentPage
                                    lastPage
                                    total
                                    hasNextPage
                                }
                                nodes{
                                    date
                                    count
                                    change
                                }
                            }
                        }
                    }
                "#,
                "variables": {
                    "manga_page": page_number
                }
            });
        } else if random_type.as_str() == "anime" {
            query = json!({
                "query": r#"
                    query($anime_page: Int = 1444){
                        SiteStatistics{
                            anime(perPage: 1, page: $anime_page){
                                pageInfo{
                                    currentPage
                                    lastPage
                                    total
                                    hasNextPage
                                }
                                nodes{
                                    date
                                    count
                                    change
                                }
                            }
                        }
                    }
                "#,
                "variables": {
                    "anime_page": page_number
                }
            });
        } else {
            return;
        }
        let res = client
            .post("https://graphql.anilist.co")
            .json(&query)
            .send()
            .await
            .unwrap()
            .json::<Value>()
            .await
            .unwrap();

        let has_next_page = res["data"]["SiteStatistics"]["manga"]["pageInfo"]["hasNextPage"]
            .as_bool()
            .unwrap_or(true);

        if !has_next_page {
            break;
        }

        cached_response = res.to_string();
        previous_page = page_number;

        sqlx::query("INSERT OR REPLACE INTO cache_stats (key, response, last_updated, last_page) VALUES (?, ?, ?, ?)")
            .bind(random_type)
            .bind(&cached_response)
            .bind(now)
            .bind(previous_page)
            .execute(&pool)
            .await.unwrap();
        embed(previous_page, random_type.to_string(), ctx, command).await;

        page_number += 1;
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
