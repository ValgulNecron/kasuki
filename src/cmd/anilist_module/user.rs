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

use crate::cmd::anilist_module::struct_user::*;
use crate::cmd::error::common::custom_error;
use crate::cmd::error::no_lang_error::{error_cant_read_langage_file, error_langage_file_not_found, error_no_langage_guild_id, error_parsing_langage_json, no_langage_error};

use crate::cmd::general_module::get_guild_langage::get_guild_langage;
use crate::cmd::general_module::lang_struct::UserLocalisedText;
use crate::cmd::general_module::pool::get_pool;

pub async fn run(
    _options: &[CommandDataOption],
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) {
    let color = Colour::FABLED_PINK;
    if let Some(option) = _options.get(0) {
        let resolved = option.resolved.as_ref().unwrap();
        if let CommandDataOptionValue::String(user) = resolved {
            let result = embed(_options, ctx, command, &user).await;
            result
        } else {
            custom_error(color, ctx, command, &"error".to_string()).await;
            return;
        }
    } else {
        let database_url = "./data.db";
        let pool = get_pool(database_url).await;
        let user_id = &command.user.id.to_string();
        let row: (Option<String>, Option<String>) = sqlx::query_as(
            "SELECT anilist_username, user_id FROM registered_user WHERE user_id = ?",
        )
        .bind(user_id)
        .fetch_one(&pool)
        .await
        .unwrap_or((None, None));
        let (user, _): (Option<String>, Option<String>) = row;
        let result = embed(
            _options,
            ctx,
            command,
            &user.unwrap_or("N/A".parse().unwrap()),
        )
        .await;
        result
    };
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("user")
        .description("Info of an anilist user")
        .create_option(|option| {
            option
                .name("username")
                .description("Username of the anilist user you want to check")
                .kind(CommandOptionType::String)
                .required(false)
                .set_autocomplete(true)
        })
}

pub async fn embed(
    _options: &[CommandDataOption],
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    value: &String,
) {
    let color = Colour::FABLED_PINK;
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

    let mut file = match File::open("lang_file/embed/anilist/user.json") {
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

    let json_data: HashMap<String, UserLocalisedText> = match serde_json::from_str(&json) {
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
        let user_url = data.get_user_url();

        let color = data.get_color();

        let chap = data.get_manga_chapter();
        let manga_genre = data.get_manga_genre();
        let manga_count = data.get_manga_count();
        let manga_score = data.get_manga_score();
        let manga_standard_deviation = data.get_manga_standard_deviation();
        let manga_tag_name = data.get_manga_tag();
        let manga_completed = data.get_manga_completed();

        let time_watched = data.time_anime_watched(localised_text.clone());

        let anime_count = data.get_anime_count();
        let anime_score = data.get_anime_score();
        let anime_standard_deviation = data.get_anime_standard_deviation();
        let anime_tag_name = data.get_anime_tag();
        let anime_genre = data.get_anime_genre();
        let anime_completed = data.get_anime_completed();

        let manga_url = data.get_user_manga_url();
        let anime_url = data.get_user_anime_url();

        let user = data.get_username();
        let profile_picture = data.get_pfp();
        let banner = data.get_banner();

        if let Err(why) = command
            .create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| {
                        message.embed(|m| {
                            m.title(user)
                                .url(&user_url)
                                // Add a timestamp for the current time
                                // This also accepts a rfc3339 Timestamp
                                .timestamp(Timestamp::now())
                                .thumbnail(profile_picture)
                                .image(banner)
                                .fields(vec![
                                    (
                                        "".to_string(),
                                        format!(
                                            "**[{}]({})**{}{}{}{}{}{}{}{:.2}{}{:.2}{}{}{}{}",
                                            &localised_text.manga_title,
                                            manga_url,
                                            &localised_text.manga_count,
                                            manga_count,
                                            &localised_text.manga_completed,
                                            manga_completed,
                                            &localised_text.manga_chapter_read,
                                            chap,
                                            &localised_text.manga_mean_score,
                                            manga_score,
                                            &localised_text.manga_standard_deviation,
                                            manga_standard_deviation,
                                            &localised_text.manga_pref_tag,
                                            manga_tag_name,
                                            &localised_text.manga_pref_genre,
                                            manga_genre
                                        ),
                                        false,
                                    ),
                                    (
                                        "".to_string(),
                                        format!(
                                            "**[{}]({})**{}{}{}{}{}{}{}{:.2}{}{:.2}{}{}{}{}",
                                            &localised_text.anime_title,
                                            anime_url,
                                            &localised_text.anime_count,
                                            anime_count,
                                            &localised_text.anime_completed,
                                            anime_completed,
                                            &localised_text.anime_time_watch,
                                            time_watched,
                                            &localised_text.anime_mean_score,
                                            anime_score,
                                            &localised_text.anime_standard_deviation,
                                            anime_standard_deviation,
                                            &localised_text.anime_pref_tag,
                                            anime_tag_name,
                                            &localised_text.anime_pref_genre,
                                            anime_genre
                                        ),
                                        false,
                                    ),
                                ])
                                .color(color)
                        })
                    })
            })
            .await
        {
            println!("{}: {}", localised_text.error_slash_command, why);
        }
    } else {
        no_langage_error(color, ctx, command).await
    }
}
