use std::u32;

use regex::Regex;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use serenity::builder::CreateApplicationCommand;
use serenity::client::Context;
use serenity::model::application::interaction::application_command::CommandDataOptionValue;
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::interaction::application_command::{
    ApplicationCommandInteraction, CommandDataOption,
};
use serenity::model::Timestamp;
use serenity::utils::Colour;

use crate::cmd::anilist_module::command_media_ln::embed;
use crate::cmd::anilist_module::struct_media::*;

const QUERY: &str = "
    query ($search: String, $limit: Int = 5) {
		Media (search: $search, type: MANGA, format: NOVEL){
    id
      description
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
    favourites
    siteUrl
    staff(perPage: $limit) {
      edges {
        node {
          id
          name {
            full
            userPreferred
          }
        }
        id
        role
      }
    }
  }
}
";

pub async fn run(
    options: &[CommandDataOption],
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) -> String {
    return embed(options, ctx, command, QUERY).await;
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("ln")
        .description("Info of a light novel")
        .create_option(|option| {
            option
                .name("ln_name")
                .description("Name of the light novel you want to check")
                .kind(CommandOptionType::String)
                .required(true)
        })
}
