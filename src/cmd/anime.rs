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

const QUERY: &str = "
query ($name: String, $limit: Int = 1) {
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
        .expect("Expected name option")
        .resolved
        .as_ref()
        .expect("Expected name object");
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
        let data: Data = serde_json::from_str(&resp.unwrap()).unwrap();
        let user_url = format!("https://anilist.co/user/{}", &data.data.User.id);
        let mut color = Colour::FABLED_PINK;
        match data.data.User.options.profileColor.as_str() {
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
        let mut min = data.data.User.statistics.anime.minutesWatched;
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
        let time_watched = format!("{} week(s), {} day(s), {} hour(s), {} minute(s)", week, days, hour, min);
        if let Err(why) = command
            .create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| message.embed(
                        |m| {
                            m.title(&data.data.User.name)
                                .url(&user_url)
                                // Add a timestamp for the current time
                                // This also accepts a rfc3339 Timestamp
                                .timestamp(Timestamp::now())
                                .thumbnail(data.data.User.avatar.large)
                                .image(data.data.User.bannerImage)
                                .fields(vec![
                                    ("Manga", format!("Count: {}\nChapters read: {}\nMean score: {:.2}\nStandard deviation: {:.2}\nPreferred tag: {}\nPreferred genre: {}",
                                                      data.data.User.statistics.manga.count,
                                                      data.data.User.statistics.manga.chaptersRead,
                                                      data.data.User.statistics.manga.meanScore,
                                                      data.data.User.statistics.manga.standardDeviation,
                                                      data.data.User.statistics.manga.tags[0].tag.name,
                                                      data.data.User.statistics.manga.genres[0].genre
                                    ), false),
                                    ("Anime", format!("Count: {}\nTime watched: {}\nMean score: {:.2}\nStandard deviation: {:.2}\nPreferred tag: {}\nPreferred genre: {}",
                                                      data.data.User.statistics.anime.count,
                                                      time_watched,
                                                      data.data.User.statistics.anime.meanScore,
                                                      data.data.User.statistics.anime.standardDeviation,
                                                      data.data.User.statistics.anime.tags[0].tag.name,
                                                      data.data.User.statistics.anime.genres[0].genre
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
    }
    return "good".to_string();
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("anime").description("Info of an anime").create_option(
        |option| {
            option
                .name("animename")
                .description("Name of the anime you want to check")
                .kind(CommandOptionType::String)
                .required(true)
        },
    )
}