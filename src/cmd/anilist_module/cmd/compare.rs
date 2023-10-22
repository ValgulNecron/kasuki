use std::cmp::Ordering;
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

use crate::cmd::anilist_module::structs::user::struct_autocomplete_user::UserPageWrapper;
use crate::cmd::anilist_module::structs::user::struct_user::*;
use crate::cmd::error_modules::common::custom_error;
use crate::cmd::error_modules::no_lang_error::{
    error_cant_read_langage_file, error_langage_file_not_found, error_no_langage_guild_id,
    error_parsing_langage_json, no_langage_error,
};
use crate::cmd::general_module::function::get_guild_langage::get_guild_langage;
use crate::cmd::general_module::lang_struct::CompareLocalisedText;
use crate::cmd::lang_struct::register::anilist::struct_compare_register::RegisterLocalisedCompare;

pub async fn run(
    options: &[CommandDataOption],
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) {
    let option = options
        .get(0)
        .expect("Expected username option")
        .resolved
        .as_ref()
        .expect("Expected username object");
    let option2 = options
        .get(1)
        .expect("Expected username option")
        .resolved
        .as_ref()
        .expect("Expected username object");
    if let CommandDataOptionValue::String(username1) = option {
        if let CommandDataOptionValue::String(username2) = option2 {
            embed(ctx, command, username1, username2).await;
            return;
        }
    }
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    let compares = RegisterLocalisedCompare::get_compare_register_localised().unwrap();
    let command = command
        .name("compare")
        .description("compare stats of two user")
        .create_option(|option| {
            let option = option
                .name("username")
                .description("Username of the 1st anilist user to compare")
                .kind(CommandOptionType::String)
                .required(true)
                .set_autocomplete(true);
            for manga in compares.values() {
                option
                    .name_localized(&manga.code, &manga.option1)
                    .description_localized(&manga.code, &manga.option1_desc);
            }
            option
        })
        .create_option(|option| {
            let option = option
                .name("username2")
                .description("Username of the 2nd anilist user to compare")
                .kind(CommandOptionType::String)
                .required(true)
                .set_autocomplete(true);
            for manga in compares.values() {
                option
                    .name_localized(&manga.code, &manga.option2)
                    .description_localized(&manga.code, &manga.option2_desc);
            }
            option
        });
    for manga in compares.values() {
        command
            .name_localized(&manga.code, &manga.name)
            .description_localized(&manga.code, &manga.desc);
    }
    command
}

pub async fn embed(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    value: &String,
    value2: &String,
) {
    let color = Colour::FABLED_PINK;
    let mut file = match File::open("lang_file/embed/anilist/compare.json") {
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

    let json_data: HashMap<String, CompareLocalisedText> = match serde_json::from_str(&json) {
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
        let data = if value.parse::<i32>().is_ok() {
            match UserWrapper::new_user_by_id(value.parse().unwrap()).await {
                Ok(user_wrapper) => user_wrapper,
                Err(error) => {
                    custom_error(color, ctx, command, &error).await;
                    return;
                }
            }
        } else {
            match UserWrapper::new_user_by_search(value).await {
                Ok(user_wrapper) => user_wrapper,
                Err(error) => {
                    custom_error(color, ctx, command, &error).await;
                    return;
                }
            }
        };

        let data2 = if value2.parse::<i32>().is_ok() {
            match UserWrapper::new_user_by_id(value2.parse().unwrap()).await {
                Ok(user_wrapper) => user_wrapper,
                Err(error) => {
                    custom_error(color, ctx, command, &error).await;
                    return;
                }
            }
        } else {
            match UserWrapper::new_user_by_search(value2).await {
                Ok(user_wrapper) => user_wrapper,
                Err(error) => {
                    custom_error(color, ctx, command, &error).await;
                    return;
                }
            }
        };

        let anime_count_text = match data.get_anime_count().cmp(&data2.get_anime_count()) {
            Ordering::Equal => {
                format!(
                    "{}{}{}{}",
                    data.get_username(),
                    &localised_text.connector_user_same_anime,
                    data2.get_username(),
                    &localised_text.same_anime
                )
            }
            Ordering::Less => {
                format!(
                    "{}{}{}",
                    data2.get_username(),
                    &localised_text.more_anime,
                    data.get_username()
                )
            }
            Ordering::Greater => {
                format!(
                    "{}{}{}",
                    data.get_username(),
                    &localised_text.more_anime,
                    data.get_username()
                )
            }
        };

        let anime_watch_time = match data.get_anime_minute().cmp(&data2.get_anime_minute()) {
            Ordering::Equal => {
                format!(
                    "{}{}{}{}",
                    data.get_username(),
                    &localised_text.connector_user_same_time,
                    data2.get_username(),
                    &localised_text.time_anime_watch
                )
            }
            Ordering::Less => {
                format!(
                    "{}{}{}",
                    data2.get_username(),
                    &localised_text.time_anime_watch,
                    data.get_username()
                )
            }
            Ordering::Greater => {
                format!(
                    "{}{}{}",
                    data.get_username(),
                    &localised_text.time_anime_watch,
                    data2.get_username()
                )
            }
        };

        let manga_count_text = match data.get_manga_count().cmp(&data2.get_manga_count()) {
            Ordering::Equal => {
                format!(
                    "{}{}{}{}",
                    data.get_username(),
                    &localised_text.connector_user_same_manga,
                    data2.get_username(),
                    &localised_text.same_manga
                )
            }
            Ordering::Greater => {
                format!(
                    "{}{}{}",
                    data.get_username(),
                    &localised_text.more_manga,
                    data2.get_username()
                )
            }
            Ordering::Less => {
                format!(
                    "{}{}{}",
                    data2.get_username(),
                    &localised_text.more_manga,
                    data.get_username()
                )
            }
        };

        let manga_chapter_count = match data.get_manga_completed().cmp(&data2.get_manga_completed())
        {
            Ordering::Equal => {
                format!(
                    "{}{}{}{}",
                    data.get_username(),
                    &localised_text.connector_user_same_chapter,
                    data2.get_username(),
                    &localised_text.same_chapter
                )
            }
            Ordering::Greater => {
                format!(
                    "{}{}{}",
                    data.get_username(),
                    &localised_text.more_chapter,
                    data2.get_username()
                )
            }
            Ordering::Less => {
                format!(
                    "{}{}{}",
                    data2.get_username(),
                    &localised_text.more_chapter,
                    data.get_username()
                )
            }
        };

        let pref_anime_genre1 = data.get_one_anime_genre(0);
        let pref_anime_genre2 = data2.get_one_anime_genre(1);
        let pref_anime_genre_text = if pref_anime_genre1 == pref_anime_genre2 {
            format!(
                "{}{}{}{}{}",
                data.get_username(),
                &localised_text.genre_same_connector_anime,
                data2.get_username(),
                &localised_text.genre_same_prefer_anime,
                pref_anime_genre1
            )
        } else {
            format!(
                "{}{}{}{}{}{}{}",
                data.get_username(),
                &localised_text.diff_pref_genre_1_anime,
                pref_anime_genre1,
                &localised_text.diff_pref_genre_while_anime,
                data2.get_username(),
                &localised_text.diff_pref_genre_2_anime,
                pref_anime_genre2
            )
        };

        let pref_anime_tag1 = data.get_one_anime_tag(0);
        let pref_anime_tag2 = data2.get_one_anime_tag(0);
        let pref_anime_tag_text = if pref_anime_tag1 == pref_anime_tag2 {
            format!(
                "{}{}{}{}{}",
                data.get_username(),
                &localised_text.same_tag_connector_anime,
                data2.get_username(),
                &localised_text.same_tag_prefer_anime,
                pref_anime_tag1
            )
        } else {
            format!(
                "{}{}{}{}{}{}{}",
                data.get_username(),
                &localised_text.diff_pref_tag_1_anime,
                pref_anime_tag1,
                &localised_text.diff_pref_tag_while_anime,
                data2.get_username(),
                &localised_text.diff_pref_tag_2_anime,
                pref_anime_tag2
            )
        };

        let pref_manga_genre1 = data.get_one_manga_genre(0);
        let pref_manga_genre2 = data2.get_one_manga_genre(0);
        let pref_manga_genre_text = if pref_manga_genre1 == pref_manga_genre2 {
            format!(
                "{}{}{}{}{}",
                data.get_username(),
                &localised_text.genre_same_connector_manga,
                data2.get_username(),
                &localised_text.genre_same_prefer_manga,
                pref_manga_genre1
            )
        } else {
            format!(
                "{}{}{}{}{}{}{}",
                data.get_username(),
                &localised_text.diff_pref_genre_1_manga,
                pref_manga_genre1,
                &localised_text.diff_pref_genre_while_manga,
                data2.get_username(),
                &localised_text.diff_pref_genre_2_manga,
                pref_manga_genre2
            )
        };

        let pref_manga_tag1 = data.get_one_manga_tag(0);
        let pref_manga_tag2 = data2.get_one_manga_tag(0);
        let pref_manga_tag_text = if pref_manga_tag1 == pref_manga_tag2 {
            format!(
                "{}{}{}{}{}",
                data.get_username(),
                &localised_text.same_tag_connector_manga,
                data2.get_username(),
                &localised_text.same_tag_prefer_manga,
                pref_manga_tag1
            )
        } else {
            format!(
                "{}{}{}{}{}{}{}",
                data.get_username(),
                &localised_text.diff_pref_tag_1_manga,
                pref_manga_tag1,
                &localised_text.diff_pref_tag_while_manga,
                data2.get_username(),
                &localised_text.diff_pref_tag_2_manga,
                pref_manga_tag2
            )
        };

        if let Err(why) = command
            .create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| {
                        message.embed(|m| {
                            m.title("Comparison")
                                // Add a timestamp for the current time
                                // This also accepts a rfc3339 Timestamp
                                .timestamp(Timestamp::now())
                                .field(
                                    "",
                                    format!(
                                        "{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}",
                                        &localised_text.sub_title_anime,
                                        anime_count_text,
                                        &localised_text.watch_time,
                                        anime_watch_time,
                                        &localised_text.pref_genre_anime,
                                        pref_anime_genre_text,
                                        &localised_text.pref_tag_anime,
                                        pref_anime_tag_text,
                                        &localised_text.sub_title_manga,
                                        manga_count_text,
                                        &localised_text.chapter_read,
                                        manga_chapter_count,
                                        &localised_text.pref_genre_manga,
                                        pref_manga_genre_text,
                                        &localised_text.pref_tag_manga,
                                        pref_manga_tag_text
                                    ),
                                    false,
                                )
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

pub async fn autocomplete(ctx: Context, command: AutocompleteInteraction) {
    // Create an empty vector to store the choices
    let mut choices = Vec::new();

    // Get the first autocomplete option and add to choices
    if let Some(option1) = command.data.options.get(0) {
        if let Some(value1) = &option1.value {
            let data1 = UserPageWrapper::new_autocomplete_user(value1, 8).await;
            choices.extend(data1.get_choice());
        }
    }

    // Get the second autocomplete option and add to choices
    if let Some(option2) = command.data.options.get(1) {
        if let Some(value2) = &option2.value {
            let data2 = UserPageWrapper::new_autocomplete_user(value2, 8).await;
            choices.extend(data2.get_choice());
        }
    }

    // Create a single autocomplete response with the collected choices
    let choices_json = json!(choices);
    _ = command
        .create_autocomplete_response(ctx.http.clone(), |response| {
            response.set_choices(choices_json)
        })
        .await;
}
