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
    data: MediaData,
}

#[derive(Debug, Deserialize)]
struct MediaData {
    Media: Media,
}

#[derive(Debug, Deserialize)]
struct Media {
    id: i64,
    title: Title,
    r#type: String,
    format: String,
    source: String,
    isAdult: bool,
    startDate: StartEndDate,
    endDate: StartEndDate,
    chapters: Option<i32>,
    volumes: Option<i32>,
    status: String,
    season: Option<String>,
    isLicensed: bool,
    coverImage: CoverImage,
    bannerImage: Option<String>,
    genres: Vec<String>,
    tags: Vec<Tag>,
    averageScore: i32,
    meanScore: i32,
    popularity: i32,
    trending: i32,
    favourites: i32,
    siteUrl: String,
}

#[derive(Debug, Deserialize)]
struct Title {
    romaji: String,
    english: String,
}

#[derive(Debug, Deserialize)]
struct StartEndDate {
    year: Option<i32>,
    month: Option<i32>,
    day: Option<i32>,
}

#[derive(Debug, Deserialize)]
struct CoverImage {
    extraLarge: String,
}

#[derive(Debug, Deserialize)]
struct Tag {
    name: String,
}

const QUERY: &str = "
    query ($search: String) {
	Media (search: $search, type: MANGA, format: NOVEL){
    id
    title{
      romaji
      english
    }
    type
    format
    source
    isAdult
    startDate {
      year
      month
      day
    }
    endDate {
      year
      month
      day
    }
    chapters
    volumes
    status
    season
    isLicensed
    coverImage {
      extraLarge
    }
    bannerImage
    genres
    tags {
      name
    }
    averageScore
    meanScore
    popularity
    trending
    favourites
    siteUrl
  }
}
";

pub async fn run(options: &[CommandDataOption], ctx: &Context, command: &ApplicationCommandInteraction) -> String {
    let option = options
        .get(0)
        .expect("Expected name option")
        .resolved
        .as_ref()
        .expect("Expected name object");
    if let CommandDataOptionValue::String(name) = option {
        let client = Client::new();
        let json = json!({"query": QUERY, "variables": {"search": name}});
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
        let data: Data = serde_json::from_str(&resp.unwrap()).unwrap();
        let hex_code = "#0D966D";
        let color_code = u32::from_str_radix(&hex_code[1..], 16).unwrap();
        let color = Colour::new(color_code);

        let name = format!("{}/{}", data.data.Media.title.english, data.data.Media.title.romaji);
        if let Err(why) = command
            .create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| message.embed(
                        |m| {
                            m.title(name)
                                .url(&data.data.Media.siteUrl)
                                .timestamp(Timestamp::now())
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
    command.name("manga").description("Info of a manga").create_option(
        |option| {
            option
                .name("manganame")
                .description("Name of the manga you want to check")
                .kind(CommandOptionType::String)
                .required(true)
        },
    )
}