use std::env;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::u32;

use chrono::Utc;
use rand::prelude::*;
use rand::rngs::StdRng;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use serenity::builder::CreateApplicationCommand;
use serenity::client::Context;
use serenity::model::application::component::ButtonStyle;
use serenity::model::application::interaction::application_command::CommandDataOptionValue;
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::prelude::ChannelId;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::interaction::application_command::{ApplicationCommandInteraction, CommandDataOption};
use serenity::model::Timestamp;
use serenity::utils::Colour;
use sqlx::{Row, SqlitePool};

use crate::cmd::anilist_module::struct_random::*;
use crate::cmd::general_module::request::make_request;

pub async fn run(options: &[CommandDataOption], ctx: &Context, command: &ApplicationCommandInteraction) -> String {
    let database_url = "./cache.db";
    let pool = match SqlitePool::connect(&database_url).await {
        Ok(pool) => pool,
        Err(e) => {
            eprintln!("Failed to connect to the database: {}", e);
            return "Error: Failed to connect to the database.".to_string();
        }
    };

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS cache_stats (
            key TEXT PRIMARY KEY,
            response TEXT NOT NULL,
            last_updated INTEGER NOT NULL,
            last_page INTEGER NOT NULL
        )",
    )
        .execute(&pool)
        .await.unwrap();

    let option = options
        .get(0)
        .expect("Expected username option")
        .resolved
        .as_ref()
        .expect("Expected username object");

    if let CommandDataOptionValue::String(random_type) = option {
        if let Err(why) = command
            .create_interaction_response(&ctx.http, |response| {
                response.kind(InteractionResponseType::DeferredChannelMessageWithSource)
            })
            .await
        {
            println!("Cannot respond to slash command: {}", why);
        }
        let row: (Option<String>, Option<i64>, Option<i64>) = sqlx::query_as("SELECT response, last_updated, last_page FROM cache_stats WHERE key = ?")
            .bind(random_type)
            .fetch_one(&pool)
            .await.unwrap_or((None, None, None));

        let (response, last_updated, last_page): (Option<String>, Option<i64>, Option<i64>) = row;

        let mut page_number = match last_page {
            Some(page) => page,
            None => 1444, // This is as today date the last page, i will update it sometime.
        };

        let mut previous_page = page_number - 1;
        let mut cached_response = response.unwrap_or("Nothing".to_string());

        let now = Utc::now().timestamp();
        if let Some(updated) = last_updated {
            let duration_since_updated = Utc::now().timestamp() - updated;
            if duration_since_updated < 24 * 60 * 60 {
                embed(cached_response, page_number, random_type.to_string(), options, ctx, command)
                    .await;
            } else {
                loop {
                    let client = Client::new();
                    let query = json!({
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
                    if random_type.as_str() == "manga" {
                        let query = json!({
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
                        let query = json!({
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
                    }


                    let res = client.post("https://graphql.anilist.co")
                        .json(&query)
                        .send()
                        .await
                        .unwrap()
                        .json::<Value>()
                        .await
                        .unwrap();

                    let has_next_page = res["data"]["SiteStatistics"]["manga"]["pageInfo"]["hasNextPage"].as_bool().unwrap_or(true);
                    let last_page = res["data"]["SiteStatistics"]["manga"]["pageInfo"]["lastPage"].as_i64().unwrap_or(page_number);

                    if !has_next_page {
                        break;
                    }

                    cached_response = res.to_string();
                    previous_page = page_number;
                    page_number += 1;
                }

                sqlx::query("INSERT OR REPLACE INTO cache_stats (key, response, last_updated, last_page) VALUES (?, ?, ?, ?)")
                    .bind(random_type)
                    .bind(&cached_response)
                    .bind(now)
                    .bind(previous_page)
                    .execute(&pool)
                    .await.unwrap();
                embed(cached_response, previous_page, random_type.to_string(), options, ctx, command)
                    .await;
            }
        } else {
            loop {
                let client = Client::new();
                let query = json!({
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
                if random_type.as_str() == "manga" {
                    let query = json!({
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
                    let query = json!({
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
                }


                let res = client.post("https://graphql.anilist.co")
                    .json(&query)
                    .send()
                    .await
                    .unwrap()
                    .json::<Value>()
                    .await
                    .unwrap();

                let has_next_page = res["data"]["SiteStatistics"]["manga"]["pageInfo"]["hasNextPage"].as_bool().unwrap_or(true);
                let last_page = res["data"]["SiteStatistics"]["manga"]["pageInfo"]["lastPage"].as_i64().unwrap_or(page_number);

                if !has_next_page {
                    break;
                }

                cached_response = res.to_string();
                previous_page = page_number;
                page_number += 1;
            }

            sqlx::query("INSERT OR REPLACE INTO cache_stats (key, response, last_updated, last_page) VALUES (?, ?, ?, ?)")
                .bind(random_type)
                .bind(&cached_response)
                .bind(now)
                .bind(previous_page)
                .execute(&pool)
                .await.unwrap();
            embed(cached_response, previous_page, random_type.to_string(), options, ctx, command)
                .await;
        }
    }
    return "good".to_string();
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("random").description("Get a random manga or anime").create_option(
        |option| {
            option
                .name("type")
                .description("Type of the media you want manga or anime. manga include ln atm.")
                .kind(CommandOptionType::String)
                .required(true)
                .add_string_choice("manga", "manga")
                .add_string_choice("anime", "anime")
        },
    )
}

pub async fn embed(res: String, last_page: i64, random_type: String, options: &[CommandDataOption], ctx: &Context, command: &ApplicationCommandInteraction) {
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
        let tags_str = tag.into_iter().map(|tag| tag.name.clone()).collect::<Vec<String>>().join("/");

        let cover_image = &media.cover_image.extra_large;

        let description = &media.description;
        let desc_no_br = description.replace("<br>", "");

        let format = &media.format;

        let url = format!("https://anilist.co/manga/{}", &media.id);
        if let Err(why) = command
            .create_followup_message(&ctx.http, |f| {
                f.embed(
                    |m| {
                        m.title(format!("{}/{}", title_user, title))
                            .description(format!("Genre: {}. \n Tags: {}. \n Format: {}. \n \n \n Description: {}"
                                                 , genres_str, tags_str, format, desc_no_br))
                            .timestamp(Timestamp::now())
                            .color(color)
                            .thumbnail(cover_image)
                            .url(url)
                    }
                )
            })
            .await
        {
            println!("Cannot respond to slash command: {}", why);
        }
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
        let res = client.post("https://graphql.anilist.co")
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
        let tags_str = tag.into_iter().map(|tag| tag.name.clone()).collect::<Vec<String>>().join("/");

        let cover_image = &media.cover_image.extra_large;

        let description = &media.description;
        let desc_no_br = description.replace("<br>", "");

        let format = &media.format;

        let url = format!("https://anilist.co/anime/{}", &media.id);
        if let Err(why) = command
            .create_followup_message(&ctx.http, |f| {
                f.embed(
                    |m| {
                        m.title(format!("{}/{}", title_user, title))
                            .description(format!("Genre: {}. \n Tags: {}. \n Format: {}. \n \n \n Description: {}"
                                                 , genres_str, tags_str, format, desc_no_br))
                            .timestamp(Timestamp::now())
                            .color(color)
                            .thumbnail(cover_image)
                            .url(url)
                    }
                )
            })
            .await
        {
            println!("Cannot respond to slash command: {}", why);
        }
    } else {
        if let Err(why) = command
            .create_followup_message(&ctx.http, |f| {
                f.embed(
                    |m| {
                        m.title("You fucked up.")
                            .description("How the heck did you managed this ?")
                            .timestamp(Timestamp::now())
                            .color(color)
                    }
                )
            })
            .await
        {
            println!("Cannot respond to slash command: {}", why);
        }
    }
}


