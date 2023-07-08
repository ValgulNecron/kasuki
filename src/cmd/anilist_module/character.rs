use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::u32;

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
use serenity::model::prelude::ChannelId;
use serenity::model::Timestamp;
use serenity::utils::Colour;

use crate::cmd::anilist_module::struct_character::*;
use crate::cmd::general_module::get_guild_langage::get_guild_langage;
use crate::cmd::general_module::lang_struct::CharacterLocalisedText;
use crate::cmd::general_module::request::make_request;

const QUERY: &str = "
query ($name: String) {
	Character(search: $name) {
    id
    name {
      full
      native
      userPreferred
    }
    siteUrl
    description
    gender
    age
    dateOfBirth {
      year
      month
      day
    }
    image {
      large
    }
    favourites
    modNotes
  }
}
";

pub async fn run(
    options: &[CommandDataOption],
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) -> String {
    let option = options
        .get(0)
        .expect("Expected username option")
        .resolved
        .as_ref()
        .expect("Expected username object");
    if let CommandDataOptionValue::String(name) = option {
        let mut file = File::open("lang_file/anilist/character.json").expect("Failed to open file");
        let mut json = String::new();
        file.read_to_string(&mut json).expect("Failed to read file");

        let json_data: HashMap<String, CharacterLocalisedText> =
            serde_json::from_str(&json).expect("Failed to parse JSON");

        let guild_id = command.guild_id.unwrap().0.to_string().clone();
        let lang_choice = get_guild_langage(guild_id).await;

        if let Some(localised_text) = json_data.get(lang_choice.as_str()) {
            let client = Client::new();
            let json = json!({"query": QUERY, "variables": {"name": name}});
            let resp = make_request(json).await;

            let data: CharacterData = serde_json::from_str(&resp).unwrap();
            let color = Colour::FABLED_PINK;

            let name = format!(
                "{}/{}",
                data.data.character.name.user_preferred, data.data.character.name.native
            );
            let desc = data.data.character.description;

            let image = data.data.character.image.large;
            let url = data.data.character.site_url;

            let age = data.data.character.age;
            let date_of_birth = format!(
                "{}/{}/{}",
                data.data.character.date_of_birth.month.unwrap_or_else(|| 0),
                data.data.character.date_of_birth.day.unwrap_or_else(|| 0),
                data.data.character.date_of_birth.year.unwrap_or_else(|| 0)
            );
            let gender = data.data.character.gender;
            let favourite = data.data.character.favourites;

            let full_description = format!(
                "{}{}{}{}{}{}{}{}{}{}.",
                &localised_text.age,
                age,
                &localised_text.gender,
                gender,
                &localised_text.date_of_birth,
                date_of_birth,
                &localised_text.favourite,
                favourite,
                &localised_text.desc,
                desc
            );

            if let Err(why) = command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| {
                            message.embed(|m| {
                                m.title(name)
                                    .url(url)
                                    .timestamp(Timestamp::now())
                                    .color(color)
                                    .description(full_description)
                                    .thumbnail(image)
                                    .color(color)
                            })
                        })
                })
                .await
            {
                println!("Cannot respond to slash command: {}", why);
            }
        }
    }
    return "good".to_string();
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("character")
        .description("Get information ")
        .create_option(|option| {
            option
                .name("name")
                .description("The name of the character")
                .kind(CommandOptionType::String)
                .required(true)
        })
}
