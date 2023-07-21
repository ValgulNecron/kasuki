use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use serde_json::json;
use serenity::builder::CreateApplicationCommand;
use serenity::client::Context;
use serenity::model::application::interaction::application_command::CommandDataOptionValue;
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::prelude::autocomplete::AutocompleteInteraction;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::interaction::application_command::{
    ApplicationCommandInteraction, CommandDataOption,
};
use serenity::model::Timestamp;
use serenity::utils::Colour;

use crate::cmd::anilist_module::struct_autocomplete::AutocompleteAnimeOption;
use crate::cmd::anilist_module::struct_autocomplete_character::CharacterPageWrapper;
use crate::cmd::anilist_module::struct_character::*;
use crate::cmd::general_module::get_guild_langage::get_guild_langage;
use crate::cmd::general_module::html_parser::convert_to_markdown;
use crate::cmd::general_module::lang_struct::CharacterLocalisedText;
use crate::cmd::general_module::request::make_request;

const QUERY_ID: &str = "
query ($name: Int) {
	Character(id: $name) {
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

const QUERY_STRING: &str = "
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
    if let CommandDataOptionValue::String(value) = option {
        let mut file = File::open("lang_file/anilist/character.json").expect("Failed to open file");
        let mut json = String::new();
        file.read_to_string(&mut json).expect("Failed to read file");

        let json_data: HashMap<String, CharacterLocalisedText> =
            serde_json::from_str(&json).expect("Failed to parse JSON");

        let guild_id = command.guild_id.unwrap().0.to_string().clone();
        let lang_choice = get_guild_langage(guild_id).await;

        if let Some(localised_text) = json_data.get(lang_choice.as_str()) {
            let query;
            if match value.parse::<i32>() {
                Ok(_) => true,
                Err(_) => false,
            } {
                query = QUERY_ID
            } else {
                query = QUERY_STRING
            }
            let json = json!({"query": query, "variables": {"name": value}});
            let resp = make_request(json).await;

            let data: CharacterData = serde_json::from_str(&resp).unwrap();
            let color = Colour::FABLED_PINK;

            let name = format!(
                "{}/{}",
                data.data.character.name.user_preferred, data.data.character.name.native
            );
            let mut desc = data.data.character.description;

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

            desc = convert_to_markdown(desc);

            let mut full_description = format!(
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

            let lenght_diff = 4096 - full_description.len() as i32;
            if lenght_diff <= 0 {
                let trim_length = desc.len() - ((lenght_diff * -1) as usize + 3);
                let desc_trim = format!("{}...", &desc[..trim_length]);
                full_description = format!(
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
                    desc_trim
                );
            }

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
                .set_autocomplete(true)
        })
}

pub async fn autocomplete(ctx: Context, command: AutocompleteInteraction) {
    let search = &command.data.options.first().unwrap().value;
    if let Some(search) = search {
        let query_str = "query ($search: String, $count: Int) {
  Page(perPage: $count) {
    characters(search: $search) {
      id
      name {
      	full
      }
    }
  }
}
";
        let json = json!({"query": query_str, "variables": {
            "search": search,
            "count": 8,
        }});
        let res = make_request(json).await;
        let data: CharacterPageWrapper = serde_json::from_str(&res).unwrap();

        if let Some(character) = data.data.page.characters {
            let suggestions: Vec<AutocompleteAnimeOption> = character
                .iter()
                .filter_map(|item| {
                    if let Some(item) = item {
                        Some(AutocompleteAnimeOption {
                            name: match &item.name {
                                Some(title) => {
                                    let english = title.user_preferred.clone();
                                    let romaji = title.full.clone();
                                    String::from(english.unwrap_or(romaji))
                                }
                                None => String::default(),
                            },
                            value: item.id.to_string(),
                        })
                    } else {
                        None
                    }
                })
                .collect();
            let choices = json!(suggestions);

            // doesn't matter if it errors
            _ = command
                .create_autocomplete_response(ctx.http, |response| {
                    response.set_choices(choices.clone())
                })
                .await;
        }
    }
}
