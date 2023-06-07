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
    statuses: Vec<Statuses>
}

#[derive(Debug, Deserialize)]
struct Manga {
    count: Option<i32>,
    meanScore: Option<f64>,
    standardDeviation: Option<f64>,
    chaptersRead: Option<i32>,
    tags: Vec<Tag>,
    genres: Vec<Genre>,
    statuses: Vec<Statuses>
}

#[derive(Debug, Deserialize)]
struct Statuses {
    count: i32,
    status: String
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
    if let CommandDataOptionValue::String(user) = option {
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
        let data: Data = match serde_json::from_str(&resp.unwrap()) {
            Ok(result) => result,
            Err(e) => {
                println!("Failed to parse json: {}", e);
                return "Error: Failed to retrieve user data".to_string();
            }
        };
        let profile_picture = data.data.User.avatar.large.unwrap_or_else(|| "https://imgs.search.brave.com/CYnhSvdQcm9aZe3wG84YY0B19zT2wlAuAkiAGu0mcLc/rs:fit:640:400:1/g:ce/aHR0cDovL3d3dy5m/cmVtb250Z3VyZHdh/cmEub3JnL3dwLWNv/bnRlbnQvdXBsb2Fk/cy8yMDIwLzA2L25v/LWltYWdlLWljb24t/Mi5wbmc".to_string());
        let user = data.data.User.name.unwrap_or_else(|| "N/A".to_string());
        let anime = data.data.User.statistics.anime;
        let manga = data.data.User.statistics.manga;
        let mut anime_completed: f64 = 0.0;
        let mut anime_watching: f64 = 0.0;
        let mut manga_completed: f64 = 0.0;
        let mut manga_reading: f64 = 0.0;
        for i in anime.statuses{
            if i.status == "COMPLETED"{
                anime_completed = i.count as f64;
            }
            else if i.status == "CURRENT"{
                anime_watching = i.count as f64
            }
        }
        for i in manga.statuses{
            if i.status == "COMPLETED"{
                manga_completed = i.count as f64;
            }
            else if i.status == "CURRENT"{
                manga_reading = i.count as f64
            }
        }
        let chap = manga.chaptersRead.unwrap_or_else(|| 0) as f64;
        let min = anime.minutesWatched.unwrap_or_else(|| 0) as f64;
        let input = (anime_completed * 2.0 + anime_watching * 1.5) + (manga_completed * 2.0 + manga_reading * 1.5) + ((chap * 5.0  + min) / 10.0);
        let scaling_factor = 2.0;
        let level = 5.0 * (1.0 + (input / scaling_factor)).ln();

        let mut color = Colour::FABLED_PINK;
        match data.data.User.options.profileColor.unwrap_or_else(|| "#FF00FF".to_string()).as_str() {
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

        if let Err(why) = command
            .create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| message.embed(
                        |m| {
                            m.title(user)
                                .timestamp(Timestamp::now())
                                .thumbnail(profile_picture)
                                .fields(vec![
                                    ("".to_string(),format!("Your level is : {}.\n You have a total of : {} xp."
                                                            ,level, input) , false),
                                ])
                                .color(color)
                        })
                    )
            })
            .await
        {
            println!("Cannot respond to slash command: {}", why);
        }
    }
    return "good".to_string();
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("level").description("Weeb level of a user").create_option(
        |option| {
            option
                .name("username")
                .description("Username of the anilist user you want to know the level of")
                .kind(CommandOptionType::String)
                .required(true)
        },
    )
}