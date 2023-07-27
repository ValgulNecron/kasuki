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
use crate::cmd::anilist_module::struct_autocomplete_user::UserPageWrapper;

use crate::cmd::anilist_module::struct_level::LevelSystem;
use crate::cmd::anilist_module::struct_user::*;
use crate::cmd::general_module::color::get_user_color;
use crate::cmd::general_module::get_guild_langage::get_guild_langage;
use crate::cmd::general_module::lang_struct::LevelLocalisedText;

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
        let mut file = File::open("lang_file/anilist/level.json").expect("Failed to open file");
        let mut json = String::new();
        file.read_to_string(&mut json).expect("Failed to read file");

        let json_data: HashMap<String, LevelLocalisedText> =
            serde_json::from_str(&json).expect("Failed to parse JSON");

        let guild_id = command.guild_id.unwrap().0.to_string().clone();
        let lang_choice = get_guild_langage(guild_id).await;

        if let Some(localised_text) = json_data.get(lang_choice.as_str()) {
            let data;
            if match value.parse::<i32>() {
                Ok(_) => true,
                Err(_) => false,
            } {
                data = match UserWrapper::new_anime_by_id(value.parse().unwrap()).await {
                    Ok(user_wrapper) => user_wrapper,
                    Err(error) => return error,
                }
            } else {
                data = match UserWrapper::new_anime_by_search(value).await {
                    Ok(user_wrapper) => user_wrapper,
                    Err(error) => return error,
                }
            }
            let profile_picture = data.data.user.avatar.large.clone().unwrap_or_else(|| "https://imgs.search.brave.com/CYnhSvdQcm9aZe3wG84YY0B19zT2wlAuAkiAGu0mcLc/rs:fit:640:400:1/g:ce/aHR0cDovL3d3dy5m/cmVtb250Z3VyZHdh/cmEub3JnL3dwLWNv/bnRlbnQvdXBsb2Fk/cy8yMDIwLzA2L25v/LWltYWdlLWljb24t/Mi5wbmc".to_string());
            let user = data
                .data
                .user
                .name
                .clone()
                .unwrap_or_else(|| "N/A".to_string());
            let anime = data.data.user.statistics.anime.clone();
            let manga = data.data.user.statistics.manga.clone();
            let (anime_completed, anime_watching) = get_total(anime.statuses.clone());
            let (manga_completed, manga_reading) = get_total(manga.statuses.clone());

            let chap = manga.chapters_read.unwrap_or_else(|| 0) as f64;
            let min = anime.minutes_watched.unwrap_or_else(|| 0) as f64;
            let input = (anime_completed * 2.5 + anime_watching * 1.0)
                + (manga_completed * 2.5 + manga_reading * 1.0)
                + chap * 5.0
                + (min / 5.0);

            let user_level;
            let user_progression;
            if let Some((level, level_progress, level_progress_total)) =
                LevelSystem::get_level(input)
            {
                user_level = level;
                user_progression = format!("{:.3}/{:.3}", level_progress, level_progress_total)
            } else {
                user_level = 0;
                user_progression = "0/0".to_string();
            }

            let color = get_user_color(data.clone());

            if let Err(why) = command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| {
                            message.embed(|m| {
                                m.title(user)
                                    .timestamp(Timestamp::now())
                                    .thumbnail(profile_picture)
                                    .fields(vec![(
                                        "".to_string(),
                                        format!(
                                            "{}{}{}{}{}{}{}",
                                            &localised_text.level,
                                            user_level,
                                            &localised_text.xp,
                                            input,
                                            &localised_text.progression_1,
                                            user_progression,
                                            &localised_text.progression_2
                                        ),
                                        false,
                                    )])
                                    .color(color)
                            })
                        })
                })
                .await
            {
                println!("Cannot respond to slash command: {}", why);
            }
        } else {
            return "Language not found".to_string();
        }
    }
    return "good".to_string();
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("level")
        .description("Weeb level of a user")
        .create_option(|option| {
            option
                .name("username")
                .description("Username of the anilist user you want to know the level of")
                .kind(CommandOptionType::String)
                .required(true)
                .set_autocomplete(true)
        })
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

pub async fn autocomplete(ctx: Context, command: AutocompleteInteraction) {
    let search = &command.data.options.first().unwrap().value;
    if let Some(search) = search {
        let data = UserPageWrapper::new_autocomplete_user(search, 8).await;
        let choices = data.get_choice();
        // doesn't matter if it errors
        let choices_json = json!(choices);
        _ = command
            .create_autocomplete_response(ctx.http.clone(), |response| {
                response.set_choices(choices_json)
            })
            .await;
    }
}