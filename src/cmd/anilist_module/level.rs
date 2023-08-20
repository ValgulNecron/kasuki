use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

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

use crate::cmd::anilist_module::struct_level::LevelSystem;
use crate::cmd::anilist_module::struct_user::*;
use crate::cmd::error::common::custom_error;
use crate::cmd::error::no_lang_error::{
    error_cant_read_langage_file, error_langage_file_not_found, error_no_langage_guild_id,
    error_parsing_langage_json, no_langage_error,
};
use crate::cmd::general_module::get_guild_langage::get_guild_langage;
use crate::cmd::general_module::lang_struct::LevelLocalisedText;

pub async fn run(
    options: &[CommandDataOption],
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) {
    let color = Colour::FABLED_PINK;

    let option = options
        .get(0)
        .expect("Expected username option")
        .resolved
        .as_ref()
        .expect("Expected username object");
    if let CommandDataOptionValue::String(value) = option {
        let mut file = match File::open("lang_file/embed/anilist/level.json") {
            Ok(file) => file,
            Err(_) => {
                error_langage_file_not_found(color, ctx, command).await;
                return;
            }
        };
        let mut json = String::new();
        match file.read_to_string(&mut json) {
            Ok(_) => {}
            Err(_) => error_cant_read_langage_file(color, ctx, command).await,
        }

        let json_data: HashMap<String, LevelLocalisedText> = match serde_json::from_str(&json) {
            Ok(data) => data,
            Err(_) => {
                error_parsing_langage_json(color, ctx, command).await;
                return;
            }
        };

        let guild_id = match command.guild_id {
            Some(id) => id.0.to_string(),
            None => {
                error_no_langage_guild_id(color, ctx, command).await;
                return;
            }
        };
        let lang_choice = get_guild_langage(guild_id).await;
        if let Some(localised_text) = json_data.get(lang_choice.as_str()) {
            let data;
            if match value.parse::<i32>() {
                Ok(_) => true,
                Err(_) => false,
            } {
                data = match UserWrapper::new_user_by_id(value.parse().unwrap()).await {
                    Ok(user_wrapper) => user_wrapper,
                    Err(error) => {
                        custom_error(color, ctx, command, &error).await;
                        return;
                    }
                }
            } else {
                data = match UserWrapper::new_user_by_search(value).await {
                    Ok(user_wrapper) => user_wrapper,
                    Err(error) => {
                        custom_error(color, ctx, command, &error).await;
                        return;
                    }
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

            let color = data.get_color();

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
                println!("{}: {}", localised_text.error_slash_command, why);
            }
        } else {
            no_langage_error(color, ctx, command).await;
        }
    }
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
