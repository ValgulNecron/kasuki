use std::env;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::u32;

use chrono::Utc;
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

pub async fn run(options: &[CommandDataOption], ctx: &Context, command: &ApplicationCommandInteraction) -> String {
    let database_url = "./cache";
    let pool = match SqlitePool::connect(&database_url).await {
        Ok(pool) => pool,
        Err(e) => {
            eprintln!("Failed to connect to the database: {}", e);
            return "Error: Failed to connect to the database.".to_string();
        }
    };

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS cache (
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
        let row: (Option<String>, Option<i64>, Option<i64>) = sqlx::query_as("SELECT response, last_updated, last_page FROM cache WHERE key = ?")
            .bind(random_type)
            .fetch_one(&pool)
            .await.unwrap_or((None, None, None));

        let (response, last_updated, last_page): (Option<String>, Option<i64>, Option<i64>) = row;

        let mut page_number = match last_page {
            Some(page) => page,
            None => 1444, // or whatever default starting page number you want
        };

        let mut previous_page = page_number - 1;
        let mut cached_response = response.unwrap_or("Nothing".to_string());

        let now = Utc::now().timestamp();

        if let Some(updated) = last_updated {
            let duration_since_updated = Utc::now().timestamp() - updated;
            if duration_since_updated < 24 * 60 * 60 {
                embed(cached_response, random_type.to_string(), options, ctx, command)
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
                    let last_page = res["data"]["SiteStatistics"]["manga"]["pageInfo"]["lastPage"].as_i64().unwrap_or(page_number as i64);

                    if !has_next_page {
                        break;
                    }

                    cached_response = res.to_string();
                    previous_page = page_number;
                    page_number += 1;
                }

                sqlx::query("INSERT OR REPLACE INTO cache (key, response, last_updated, last_page) VALUES (?, ?, ?, ?)")
                    .bind(random_type)
                    .bind(&cached_response)
                    .bind(now)
                    .bind(previous_page)
                    .execute(&pool)
                    .await.unwrap();
                println!("before embed no cache");
                embed(cached_response, random_type.to_string(), options, ctx, command)
                    .await;
            }
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

pub async fn embed(res: String, random_type: String, options: &[CommandDataOption], ctx: &Context, command: &ApplicationCommandInteraction) {
    let color = Colour::FABLED_PINK;

    if let Err(why) = command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| message.embed(
                    |m| {
                        m.title("Info")
                            .description("This bot use the anilist api to give information on a show or a user")
                            .footer(|f| f.text("creator valgul#8329"))
                            // Add a timestamp for the current time
                            // This also accepts a rfc3339 Timestamp
                            .timestamp(Timestamp::now())
                            .color(color)
                    })
                    .components(|components| {
                        components.create_action_row(|row| {
                            row.create_button(|button| {
                                button.label("See on github")
                                    .url("https://github.com/ValgulNecron/DIscordAnilistBotRS")
                                    .style(ButtonStyle::Link)
                            })
                                .create_button(|button| {
                                    button.label("Official website")
                                        .url("https://discord.com/api/oauth2/authorize?client_id=923286536445894697&permissions=17861158751296&scope=bot")
                                        .style(ButtonStyle::Link)
                                })
                        })
                            .create_action_row(|button| {
                                button.create_button(|button| {
                                    button.label("Official discord")
                                        .url("https://discord.gg/dWGU6mkw7J")
                                        .style(ButtonStyle::Link)
                                })
                                    .create_button(|button| {
                                        button.label("Add the bot.")
                                            .url("https://discord.com/api/oauth2/authorize?client_id=923286536445894697&permissions=2048&scope=bot")
                                            .style(ButtonStyle::Link)
                                    })
                            })
                    })
                )
        })
        .await
    {
        println!("Cannot respond to slash command: {}", why);
    }
}