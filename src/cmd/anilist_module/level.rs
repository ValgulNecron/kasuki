use std::any::Any;

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

use crate::cmd::anilist_module::struct_user::*;
use crate::cmd::general_module::color::get_user_color;
use crate::cmd::general_module::request::make_request;

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

const WIDTH: u32 = 400;
const HEIGHT: u32 = 40;

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
        let resp = make_request(json).await;
        // Get json
        let data: UserData = match resp_to_user_data(resp) {
            Ok(data) => {
                data
            }
            Err(error) => {
                return error;
            }
        };
        let profile_picture = data.data.user.avatar.large.clone().unwrap_or_else(|| "https://imgs.search.brave.com/CYnhSvdQcm9aZe3wG84YY0B19zT2wlAuAkiAGu0mcLc/rs:fit:640:400:1/g:ce/aHR0cDovL3d3dy5m/cmVtb250Z3VyZHdh/cmEub3JnL3dwLWNv/bnRlbnQvdXBsb2Fk/cy8yMDIwLzA2L25v/LWltYWdlLWljb24t/Mi5wbmc".to_string());
        let user = data.data.user.name.clone().unwrap_or_else(|| "N/A".to_string());
        let anime = data.data.user.statistics.anime.clone();
        let manga = data.data.user.statistics.manga.clone();
        let (anime_completed, anime_watching) = get_total(anime.statuses.clone());
        let (manga_completed, manga_reading) = get_total(manga.statuses.clone());

        let chap = manga.chapters_read.unwrap_or_else(|| 0) as f64;
        let min = anime.minutes_watched.unwrap_or_else(|| 0) as f64;
        let input = (anime_completed * 2.0 + anime_watching * 1.0) + (manga_completed * 2.0 + manga_reading * 1.0) + chap * 5.0 + (min / 10.0);
        let a = 5.0;
        let b = 0.000005;
        let level_float = a * (input).ln() + (b * input);
        let level = level_float.floor();

        let progress_percent = (level_float - level) * 100.0;

        let color = get_user_color(data.clone());

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
                                    ("".to_string(), format!("Your level is : {}.\n You have a total \
                                    of : {} xp. \n Your are at {}% to the next level."
                                                             , level, input, progress_percent.floor()), false),
                                ])
                                .color(color)
                        }),
                    )
            }).await
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

pub fn get_total(media: Vec<Statuses>) -> (f64, f64) {
    let mut watching = 0.0;
    let mut completed = 0.0;
    for i in media {
        if i.status == "COMPLETED".to_string() {
            completed = i.count as f64;
        } else if i.status == "CURRENT".to_string() {
            watching = i.count as f64
        }
    }
    let tuple = (watching, completed);
    return tuple;
}