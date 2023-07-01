use std::u32;

use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use serenity::builder::CreateApplicationCommand;
use serenity::client::Context;
use serenity::model::application::interaction::application_command::CommandDataOptionValue;
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::prelude::ChannelId;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::interaction::application_command::{ApplicationCommandInteraction, CommandDataOption};
use serenity::model::Timestamp;
use serenity::utils::Colour;
use sqlx::SqlitePool;

use crate::cmd::anilist_module::struct_user::*;

const QUERY: &str = "
query ($name: String, $limit: Int = 5) {
  User(name: $name) {
    id
    name
    avatar {
      large
    }
    statistics {
      anime {
        count
        meanScore
        standardDeviation
        minutesWatched
        tags(limit: $limit, sort: COUNT_DESC) {
          tag {
            name
          }
        }
        genres(limit: $limit, sort: COUNT_DESC) {
          genre
        }
        statuses(sort: COUNT_DESC){
          count
          status
        }
      }
      manga {
        count
        meanScore
        standardDeviation
        chaptersRead
        tags(limit: $limit) {
          tag {
            name
          }
        }
        genres(limit: $limit) {
          genre
        }
        statuses(sort: COUNT_DESC){
          count
          status
        }
      }
    }
options{
      profileColor
    }
    bannerImage
  }
}
";

pub async fn run(options: &[CommandDataOption], ctx: &Context, command: &ApplicationCommandInteraction) -> String {
    return if let Some(option) = options.get(0) {
        let resolved = option.resolved.as_ref().unwrap();
        if let CommandDataOptionValue::String(user) = resolved {
            let result = embed(options, ctx, command, &user).await;
            result
        } else {
            "error".to_string()
        }
    } else {
        let database_url = "./data.db";
        let pool = match SqlitePool::connect(&database_url).await {
            Ok(pool) => pool,
            Err(e) => {
                eprintln!("Failed to connect to the database: {}", e);
                return "Error: Failed to connect to the database.".to_string();
            }
        };
        let user_id = &command.user.id.to_string();
        let row: (Option<String>, Option<String>) = sqlx::query_as("SELECT anilist_username, user_id FROM registered_user WHERE user_id = ?")
            .bind(user_id)
            .fetch_one(&pool)
            .await.unwrap_or((None, None));
        let (user, _): (Option<String>, Option<String>) = row;
        let result = embed(options, ctx, command, &user.unwrap_or("N/A".parse().unwrap())).await;
        result
    };
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("user").description("Info of an anilist user").create_option(
        |option| {
            option
                .name("username")
                .description("Username of the anilist user you want to check")
                .kind(CommandOptionType::String)
                .required(false)
        },
    )
}


pub async fn embed(options: &[CommandDataOption], ctx: &Context, command: &ApplicationCommandInteraction, user: &String) -> String {
    let client = Client::new();
    let json = json!({"query": QUERY, "variables": {"name": user}});
    let resp = client.post("https://graphql.anilist.co/")
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .body(json.to_string())
        .send()
        .await
        .unwrap()
        .text()
        .await;

    // Get json
    let data: UserData = match serde_json::from_str(&resp.unwrap()) {
        Ok(result) => result,
        Err(e) => {
            println!("Failed to parse json: {}", e);
            return "Error: Failed to retrieve user data".to_string();
        }
    };
    let user_url = format!("https://anilist.co/user/{}", &data.data.user.id.unwrap_or_else(|| 1));
    let mut color = Colour::FABLED_PINK;
    match data.data.user.options.profile_color.unwrap_or_else(|| "#FF00FF".to_string()).as_str() {
        "blue" => color = Colour::BLUE,
        "purple" => color = Colour::PURPLE,
        "pink" => color = Colour::MEIBE_PINK,
        "orange" => color = Colour::ORANGE,
        "red" => color = Colour::RED,
        "green" => color = Colour::DARK_GREEN,
        "gray" => color = Colour::LIGHT_GREY,
        _ => color = {
            let hex_code = "#0D966D";
            let color_code = u32::from_str_radix(&hex_code[1..], 16).unwrap();
            Colour::new(color_code)
        },
    }
    let mut min = data.data.user.statistics.anime.minutes_watched.unwrap_or_else(|| 0);
    let mut hour = 0;
    let mut days = 0;
    let mut week = 0;

    if min >= 60 {
        hour = min / 60;
        min = min % 60;
    }

    if hour >= 24 {
        days = hour / 24;
        hour = hour % 24;
    }

    if days >= 7 {
        week = days / 7;
        days = days % 7;
    }
    let chap = data.data.user.statistics.manga.chapters_read.unwrap_or_else(|| 0);
    let time_watched = format!("{} week(s), {} day(s), {} hour(s), {} minute(s)", week, days, hour, min);
    let manga_count = data.data.user.statistics.manga.count.unwrap_or_else(|| 0);
    let manga_score = data.data.user.statistics.manga.mean_score.unwrap_or_else(|| 0 as f64);
    let manga_standard_deviation = data.data.user.statistics.manga.standard_deviation.unwrap_or_else(|| 0 as f64);
    let mut manga_tag_name = String::new();
    for i in 0..3 {
        if let Some(tags) = data.data.user.statistics.manga.tags.get(i).and_then(|g| g.tag.name.as_ref()) {
            manga_tag_name.push_str(&format!("{} / ", tags));
        } else {
            manga_tag_name.push_str("N/A / ");
        }
    }
    manga_tag_name.pop();
    manga_tag_name.pop();

    let mut manga_genre = String::new();
    for i in 0..3 {
        if let Some(genre) = data.data.user.statistics.manga.genres.get(i).and_then(|g| g.genre.as_ref()) {
            manga_genre.push_str(&format!("{} / ", genre));
        } else {
            manga_genre.push_str("N/A / ");
        }
    }
    manga_genre.pop();
    manga_genre.pop();


    let anime_count = data.data.user.statistics.anime.count.unwrap_or_else(|| 0);
    let anime_score = data.data.user.statistics.anime.mean_score.unwrap_or_else(|| 0 as f64);
    let anime_standard_deviation = data.data.user.statistics.anime.standard_deviation.unwrap_or_else(|| 0 as f64);

    let mut anime_tag_name = String::new();
    for i in 0..3 {
        if let Some(tags) = data.data.user.statistics.anime.tags.get(i).and_then(|g| g.tag.name.as_ref()) {
            anime_tag_name.push_str(&format!("{} / ", tags));
        } else {
            anime_tag_name.push_str("N/A / ");
        }
    }
    anime_tag_name.pop();
    anime_tag_name.pop();

    let mut anime_genre = String::new();
    for i in 0..3 {
        if let Some(genre) = data.data.user.statistics.anime.genres.get(i).and_then(|g| g.genre.as_ref()) {
            anime_genre.push_str(&format!("{} / ", genre));
        } else {
            anime_genre.push_str("N/A / ");
        }
    }
    anime_genre.pop();
    anime_genre.pop();

    let manga_url = format!("{}/mangalist", &user_url);
    let anime_url = format!("{}/animelist", &user_url);

    let anime_statuses = data.data.user.statistics.anime.statuses;
    let mut anime_completed = 0;
    for i in anime_statuses {
        if i.status == "COMPLETED".to_string() {
            anime_completed = i.count;
        }
    }

    let manga_statuses = data.data.user.statistics.manga.statuses;
    let mut manga_completed = 0;
    for i in manga_statuses {
        if i.status == "COMPLETED".to_string() {
            manga_completed = i.count;
        }
    }
    let user = data.data.user.name.unwrap_or_else(|| "N/A".to_string());
    let profile_picture = data.data.user.avatar.large.unwrap_or_else(|| "https://imgs.search.brave.com/CYnhSvdQcm9aZe3wG84YY0B19zT2wlAuAkiAGu0mcLc/rs:fit:640:400:1/g:ce/aHR0cDovL3d3dy5m/cmVtb250Z3VyZHdh/cmEub3JnL3dwLWNv/bnRlbnQvdXBsb2Fk/cy8yMDIwLzA2L25v/LWltYWdlLWljb24t/Mi5wbmc".to_string());
    let banner_old = data.data.user.banner_image.unwrap_or_else(|| "https://imgs.search.brave.com/CYnhSvdQcm9aZe3wG84YY0B19zT2wlAuAkiAGu0mcLc/rs:fit:640:400:1/g:ce/aHR0cDovL3d3dy5m/cmVtb250Z3VyZHdh/cmEub3JnL3dwLWNv/bnRlbnQvdXBsb2Fk/cy8yMDIwLzA2L25v/LWltYWdlLWljb24t/Mi5wbmc".to_string());
    let banner = format!("https://img.anili.st/user/{}", data.data.user.id.unwrap());

    if let Err(why) = command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| message.embed(
                    |m| {
                        m.title(user)
                            .url(&user_url)
                            // Add a timestamp for the current time
                            // This also accepts a rfc3339 Timestamp
                            .timestamp(Timestamp::now())
                            .thumbnail(profile_picture)
                            .image(banner)
                            .fields(vec![
                                ("".to_string(), format!("**[Manga]({})** \nCount: {} And completed : {}\nChapters read: {}\nMean score: {:.2}\nStandard deviation: {:.2}\nPreferred tag: {}\nPreferred genre: {}",
                                                         manga_url,
                                                         manga_count,
                                                         manga_completed,
                                                         chap,
                                                         manga_score,
                                                         manga_standard_deviation,
                                                         manga_tag_name,
                                                         manga_genre
                                ), false),
                                ("".to_string(), format!("**[Anime]({})**\nCount: {} And completed : {}\nTime watched: {}\nMean score: {:.2}\nStandard deviation: {:.2}\nPreferred tag: {}\nPreferred genre: {}",
                                                         anime_url,
                                                         anime_count,
                                                         anime_completed,
                                                         time_watched,
                                                         anime_score,
                                                         anime_standard_deviation,
                                                         anime_tag_name,
                                                         anime_genre
                                ), false),
                            ])
                            .color(color)
                    })
                )
        })
        .await
    {
        println!("Cannot respond to slash command: {}", why);
    }
    return "good".to_string();
}