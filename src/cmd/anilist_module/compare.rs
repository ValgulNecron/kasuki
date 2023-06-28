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

#[derive(Debug, Deserialize)]
struct Data {
    data: UserWrapper,
}

#[derive(Debug, Deserialize)]
struct UserWrapper {
    User: User,
}

#[derive(Debug, Deserialize)]
struct User {
    id: Option<i32>,
    name: Option<String>,
    avatar: Avatar,
    statistics: Statistics,
    options: Options,
    bannerImage: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Options {
    profileColor: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Avatar {
    large: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Statistics {
    anime: Anime,
    manga: Manga,
}

#[derive(Debug, Deserialize)]
struct Anime {
    count: Option<i32>,
    meanScore: Option<f64>,
    standardDeviation: Option<f64>,
    minutesWatched: Option<i32>,
    tags: Vec<Tag>,
    genres: Vec<Genre>,
    statuses: Vec<Statuses>,
}

#[derive(Debug, Deserialize)]
struct Manga {
    count: Option<i32>,
    meanScore: Option<f64>,
    standardDeviation: Option<f64>,
    chaptersRead: Option<i32>,
    tags: Vec<Tag>,
    genres: Vec<Genre>,
    statuses: Vec<Statuses>,
}

#[derive(Debug, Deserialize)]
struct Statuses {
    count: i32,
    status: String,
}

#[derive(Debug, Deserialize)]
struct Tag {
    tag: TagData,
}

#[derive(Debug, Deserialize)]
struct TagData {
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Genre {
    pub genre: Option<String>,
}

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
    let option = options
        .get(0)
        .expect("Expected username option")
        .resolved
        .as_ref()
        .expect("Expected username object");
    let option2 = options
        .get(1)
        .expect("Expected username option")
        .resolved
        .as_ref()
        .expect("Expected username object");
    if let CommandDataOptionValue::String(username1) = option {
        if let CommandDataOptionValue::String(username2) = option2 {
            let result = embed(options, ctx, command, username1, username2).await;
            return result
        }
    }
    return "good".to_string();
}


pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("compare").description("compare stats of two uer").create_option(
        |option| {
            option
                .name("username")
                .description("Username of the 1st anilist user to compare")
                .kind(CommandOptionType::String)
                .required(true)
        }
    ).create_option(|option| {
        option
            .name("username2")
            .description("Username of the 1st anilist user to compare")
            .kind(CommandOptionType::String)
            .required(true)
    })
}

pub async fn embed(options: &[CommandDataOption], ctx: &Context, command: &ApplicationCommandInteraction, username1: &String, username2: &String) -> String {
    let client = Client::new();
    let json = json!({"query": QUERY, "variables": {"name": username1}});
    let resp = client.post("https://graphql.anilist.co/")
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .body(json.to_string())
        .send()
        .await
        .unwrap()
        .text()
        .await;

    let client2 = Client::new();
    let json2 = json!({"query": QUERY, "variables": {"name": username2}});
    let resp2 = client2.post("https://graphql.anilist.co/")
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .body(json2.to_string())
        .send()
        .await
        .unwrap()
        .text()
        .await;

    let data: Data = match serde_json::from_str(&resp.unwrap()) {
        Ok(result) => result,
        Err(e) => {
            println!("Failed to parse json: {}", e);
            return "Error: Failed to retrieve user data".to_string();
        }
    };

    let data2: Data = match serde_json::from_str(&resp2.unwrap()) {
        Ok(result) => result,
        Err(e) => {
            println!("Failed to parse json: {}", e);
            return "Error: Failed to retrieve user data".to_string();
        }
    };

    let user1 = data.data.User;
    let user2 = data2.data.User;

    let user_name1 = user1.name.unwrap().clone();
    let user_name2 = user2.name.unwrap().clone();

    let anime_count_text;
    if user1.statistics.anime.count > user2.statistics.anime.count {
        anime_count_text = format!("{} as more anime than {}", user_name1, user_name2)
    } else if user1.statistics.anime.count < user2.statistics.anime.count {
        anime_count_text = format!("{} as more anime than {}", user_name2, user_name1)
    } else {
        anime_count_text = format!("{} and {} as the same amount of anime watched.", user_name1, user_name2)
    }

    let anime_watch_time;
    if user1.statistics.anime.minutesWatched > user2.statistics.anime.minutesWatched {
        anime_watch_time = format!("{} as watched anime for longer than {}", user_name1, user_name2)
    } else if user1.statistics.anime.minutesWatched < user2.statistics.anime.minutesWatched {
        anime_watch_time = format!("{} as watched anime for longer than {}", user_name2, user_name1)
    } else {
        anime_watch_time = format!("{} and {} as the same amount of anime watch time.", user_name1, user_name2)
    }

    let manga_count_text;
    if user1.statistics.manga.count > user2.statistics.manga.count {
        manga_count_text = format!("{} as read more manga than {}", user_name1, user_name2)
    } else if user1.statistics.manga.count < user2.statistics.manga.count {
        manga_count_text = format!("{} as read more manga than {}", user_name2, user_name1)
    } else {
        manga_count_text = format!("{} and {} as the same amount of manga read.", user_name1, user_name2)
    }

    let manga_chapter_count;
    if user1.statistics.manga.chaptersRead > user2.statistics.manga.chaptersRead {
        manga_chapter_count = format!("{} as read more chapter than {}", user_name1, user_name2)
    } else if user1.statistics.manga.chaptersRead < user2.statistics.manga.chaptersRead {
        manga_chapter_count = format!("{} as read more chapter than {}", user_name2, user_name1)
    } else {
        manga_chapter_count = format!("{} and {} as the same amount of chapter read.", user_name1, user_name2)
    }

    let color = Colour::FABLED_PINK;

    if let Err(why) = command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| message.embed(
                    |m| {
                        m.title("Comparison")
                            // Add a timestamp for the current time
                            // This also accepts a rfc3339 Timestamp
                            .timestamp(Timestamp::now())
                            .field("", format!("Anime: {}. \n Watch Time: {}. \n Manga: {}. \n Chapter read {}.",
                                           anime_count_text, anime_watch_time,
                                           manga_count_text, manga_chapter_count), false)
                            .color(color)
                    })
                )
        })
        .await
    {
        println!("Cannot respond to slash command: {}", why);
    }

    let mut color = Colour::FABLED_PINK;
    return "good".to_string();
}