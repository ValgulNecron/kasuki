use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use serde_json::json;
use serenity::builder::CreateApplicationCommand;
use serenity::client::Context;
use serenity::model::prelude::application_command::{ApplicationCommandInteraction, CommandDataOption};
use serenity::model::prelude::InteractionResponseType;
use serenity::model::Timestamp;
use serenity::utils::Colour;
use crate::cmd::anilist_module::character::QUERY_ID;
use crate::cmd::anilist_module::struct_character::CharacterWrapper;
use crate::cmd::general_module::get_guild_langage::get_guild_langage;
use crate::cmd::general_module::html_parser::convert_to_markdown;
use crate::cmd::general_module::lang_struct::CharacterLocalisedText;
use crate::cmd::general_module::request::make_request_anilist;
use crate::cmd::general_module::trim::trim;

pub async fn run(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) -> String {
let mut file = File::open("lang_file/anilist/character.json").expect("Failed to open file");
        let mut json = String::new();
        file.read_to_string(&mut json).expect("Failed to read file");

        let json_data: HashMap<String, CharacterLocalisedText> =
            serde_json::from_str(&json).expect("Failed to parse JSON");

        let guild_id = command.guild_id.unwrap().0.to_string().clone();
        let lang_choice = get_guild_langage(guild_id).await;

        if let Some(localised_text) = json_data.get(lang_choice.as_str()) {
            let query = QUERY_ID;
            let json = json!({"query": query, "variables": {"name": 156323}});
            let resp = make_request_anilist(json, false).await;

            let data: CharacterWrapper = serde_json::from_str(&resp).unwrap();
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
                desc = trim(desc, lenght_diff);

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
                    desc
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
    return "good".to_string();
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("waifu").description("Give you the best waifu.")
}