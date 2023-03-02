use reqwest::Client;
use serde_json::json;
use serenity::builder::CreateApplicationCommand;
use serenity::client::Context;
use serenity::model::application::interaction::application_command::CommandDataOptionValue;
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::prelude::ChannelId;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::interaction::application_command::{ApplicationCommandInteraction, CommandDataOption};
use serenity::model::Timestamp;
use serde::{Deserialize, Serialize};

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
    id: i32,
    name: String,
    avatar: Avatar,
    statistics: Statistics,
}

#[derive(Debug, Deserialize)]
struct Avatar {
    large: String,
}

#[derive(Debug, Deserialize)]
struct Statistics {
    anime: Anime,
    manga: Manga,
}

#[derive(Debug, Deserialize)]
struct Anime {
    count: i32,
    meanScore: f64,
    standardDeviation: f64,
    minutesWatched: i32,
    tags: Vec<Tag>,
    genres: Vec<Genre>,
}

#[derive(Debug, Deserialize)]
struct Manga {
    count: i32,
    meanScore: f64,
    standardDeviation: f64,
    chaptersRead: i32,
    tags: Vec<Tag>,
        genres: Vec<Genre>,
}

#[derive(Debug, Deserialize)]
struct Tag {
    tag: TagData,
}

#[derive(Debug, Deserialize)]
struct TagData {
    name: String,
}

#[derive(Debug, Deserialize)]
pub struct Genre {
    pub genre: String,
}

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
                let data: Data = serde_json::from_str(&resp.unwrap()).unwrap();
        let image_url = "https://s4.anilist.co/file/anilistcdn/user/avatar/large/b5399974-v6O5Klq9cY7e.jpg";
        if let Err(why) = command
            .create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| message.embed(
                        |m| {
                            m.title(data.data.User.name)
                                // Add a timestamp for the current time
                                // This also accepts a rfc3339 Timestamp
                                .timestamp(Timestamp::now())
                                .image(data.data.User.avatar.large)
                                .fields(vec![
    ("Manga", format!("Count: {}\nChapters read: {}\nMean score: {:.2}\nStandard deviation: {:.2}\nPreferred tag: {}\nPreferred genre: {}",
        data.data.User.statistics.manga.count,
        data.data.User.statistics.manga.chaptersRead,
        data.data.User.statistics.manga.meanScore,
        data.data.User.statistics.manga.standardDeviation,
        data.data.User.statistics.manga.tags[0].tag.name,
        data.data.User.statistics.manga.genres[0].genre
    ), true),
    ("Anime", format!("Count: {}\nTime watched: {} minutes\nMean score: {:.2}\nStandard deviation: {:.2}\nPreferred tag: {}\nPreferred genre: {}",
        data.data.User.statistics.anime.count,
        data.data.User.statistics.anime.minutesWatched,
        data.data.User.statistics.anime.meanScore,
        data.data.User.statistics.anime.standardDeviation,
        data.data.User.statistics.anime.tags[0].tag.name,
        data.data.User.statistics.anime.genres[0].genre
    ), true),
])



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
    command.name("user").description("Info of an anilist user").create_option(
        |option| {
            option
                .name("username")
                .description("Username of the anilist user you want to check")
                .kind(CommandOptionType::String)
                .required(true)
        },
    )
}