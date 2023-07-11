use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

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

use crate::cmd::anilist_module::struct_user::*;
use crate::cmd::general_module::get_guild_langage::get_guild_langage;
use crate::cmd::general_module::lang_struct::CompareLocalisedText;
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
        tags(limit: $limit, sort: MEAN_SCORE_DESC) {
          tag {
            name
          }
        }
        genres(limit: $limit, sort: MEAN_SCORE_DESC) {
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
        tags(limit: $limit, sort: MEAN_SCORE_DESC) {
          tag {
            name
          }
        }
        genres(limit: $limit, sort: MEAN_SCORE_DESC) {
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
    let option2 = options
        .get(1)
        .expect("Expected username option")
        .resolved
        .as_ref()
        .expect("Expected username object");
    if let CommandDataOptionValue::String(username1) = option {
        if let CommandDataOptionValue::String(username2) = option2 {
            let result = embed(ctx, command, username1, username2).await;
            return result;
        }
    }
    return "good".to_string();
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("compare")
        .description("compare stats of two uer")
        .create_option(|option| {
            option
                .name("username")
                .description("Username of the 1st anilist user to compare")
                .kind(CommandOptionType::String)
                .required(true)
        })
        .create_option(|option| {
            option
                .name("username2")
                .description("Username of the 1st anilist user to compare")
                .kind(CommandOptionType::String)
                .required(true)
        })
}

pub async fn embed(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    username1: &String,
    username2: &String,
) -> String {
    let mut file = File::open("lang_file/anilist/compare.json").expect("Failed to open file");
    let mut json = String::new();
    file.read_to_string(&mut json).expect("Failed to read file");

    let json_data: HashMap<String, CompareLocalisedText> =
        serde_json::from_str(&json).expect("Failed to parse JSON");

    let guild_id = command.guild_id.unwrap().0.to_string().clone();
    let lang_choice = get_guild_langage(guild_id).await;

    if let Some(localised_text) = json_data.get(lang_choice.as_str()) {
        let json = json!({"query": QUERY, "variables": {"name": username1}});
        let resp = make_request(json).await;

        let json2 = json!({"query": QUERY, "variables": {"name": username2}});
        let resp2 = make_request(json2).await;

        let data: UserData = match resp_to_user_data(resp) {
            Ok(data) => data,
            Err(error) => {
                return error;
            }
        };

        let data2: UserData = match resp_to_user_data(resp2) {
            Ok(data) => data,
            Err(error) => {
                return error;
            }
        };

        let user1 = data.data.user;
        let user2 = data2.data.user;

        let user_name1 = user1.name.unwrap().clone();
        let user_name2 = user2.name.unwrap().clone();

        let anime_count_text;
        if user1.statistics.anime.count > user2.statistics.anime.count {
            anime_count_text = format!("{}{}{}", user_name1, &localised_text.more_anime, user_name2)
        } else if user1.statistics.anime.count < user2.statistics.anime.count {
            anime_count_text = format!("{}{}{}", user_name2, &localised_text.more_anime, user_name1)
        } else {
            anime_count_text = format!(
                "{}{}{}{}",
                user_name1,
                &localised_text.connector_user_same_anime,
                user_name2,
                &localised_text.same_anime
            )
        }

        let anime_watch_time;
        if user1.statistics.anime.minutes_watched > user2.statistics.anime.minutes_watched {
            anime_watch_time = format!("{}{}{}", user_name1, &localised_text.time_anime_watch, user_name2)
        } else if user1.statistics.anime.minutes_watched < user2.statistics.anime.minutes_watched {
            anime_watch_time = format!("{}{}{}", user_name2, &localised_text.time_anime_watch, user_name1)
        } else {
            anime_watch_time = format!(
                "{}{}{}{}",
                user_name1,
                &localised_text.connector_user_same_time,
                user_name2,
                &localised_text.time_anime_watch
            )
        }

        let manga_count_text;
        if user1.statistics.manga.count > user2.statistics.manga.count {
            manga_count_text = format!("{}{}{}", user_name1, &localised_text.more_manga, user_name2)
        } else if user1.statistics.manga.count < user2.statistics.manga.count {
            manga_count_text = format!("{}{}{}", user_name2, &localised_text.more_manga, user_name1)
        } else {
            manga_count_text = format!(
                "{}{}{}{}",
                user_name1,
                &localised_text.connector_user_same_manga,
                user_name2,
                &localised_text.same_manga
            )
        }

        let manga_chapter_count;
        if user1.statistics.manga.chapters_read > user2.statistics.manga.chapters_read {
            manga_chapter_count = format!(
                "{}{}{}",
                user_name1, &localised_text.more_chapter, user_name2
            )
        } else if user1.statistics.manga.chapters_read < user2.statistics.manga.chapters_read {
            manga_chapter_count = format!(
                "{}{}{}",
                user_name2, &localised_text.more_chapter, user_name1
            )
        } else {
            manga_chapter_count = format!(
                "{}{}{}{}",
                user_name1,
                &localised_text.connector_user_same_chapter,
                user_name2,
                &localised_text.same_chapter
            )
        }

        let pref_anime_genre1 = user1.statistics.anime.genres[0].clone().genre.unwrap();
        let pref_anime_genre2 = user2.statistics.anime.genres[0].clone().genre.unwrap();
        let pref_anime_genre_text;
        if pref_anime_genre1 == pref_anime_genre2 {
            pref_anime_genre_text = format!(
                "{}{}{}{}{}",
                user_name1,
                &localised_text.genre_same_connector_anime,
                user_name2,
                &localised_text.genre_same_prefer_anime,
                pref_anime_genre1
            );
        } else {
            pref_anime_genre_text = format!(
                "{}{}{}{}{}{}{}",
                user_name1,
                &localised_text.diff_pref_genre_1_anime,
                pref_anime_genre1,
                &localised_text.diff_pref_genre_while_anime,
                user_name2,
                &localised_text.diff_pref_genre_2_anime,
                pref_anime_genre2
            );
        }

        let pref_anime_tag1 = user1.statistics.anime.tags[0].clone().tag.name.unwrap();
        let pref_anime_tag2 = user2.statistics.anime.tags[0].clone().tag.name.unwrap();
        let pref_anime_tag_text;
        if pref_anime_tag1 == pref_anime_tag2 {
            pref_anime_tag_text = format!(
                "{}{}{}{}{}",
                user_name1,
                &localised_text.same_tag_connector_anime,
                user_name2,
                &localised_text.same_tag_prefer_anime,
                pref_anime_tag1
            );
        } else {
            pref_anime_tag_text = format!(
                "{}{}{}{}{}{}{}",
                user_name1,
                &localised_text.diff_pref_tag_1_anime,
                pref_anime_tag1,
                &localised_text.diff_pref_tag_while_anime,
                user_name2,
                &localised_text.diff_pref_tag_2_anime,
                pref_anime_tag2
            );
        }

        let pref_manga_genre1 = user1.statistics.manga.genres[0].clone().genre.unwrap();
        let pref_manga_genre2 = user2.statistics.manga.genres[0].clone().genre.unwrap();
        let pref_manga_genre_text;
        if pref_manga_genre1 == pref_manga_genre2 {
            pref_manga_genre_text = format!(
                "{}{}{}{}{}",
                user_name1,
                &localised_text.genre_same_connector_manga,
                user_name2,
                &localised_text.genre_same_prefer_manga,
                pref_manga_genre1
            );
        } else {
            pref_manga_genre_text = format!(
                "{}{}{}{}{}{}{}",
                user_name1,
                &localised_text.diff_pref_genre_1_manga,
                pref_manga_genre1,
                &localised_text.diff_pref_genre_while_manga,
                user_name2,
                &localised_text.diff_pref_genre_2_manga,
                pref_manga_genre2
            );
        }

        let pref_manga_tag1 = user1.statistics.manga.tags[0].clone().tag.name.unwrap();
        let pref_manga_tag2 = user2.statistics.manga.tags[0].clone().tag.name.unwrap();
        let pref_manga_tag_text;
        if pref_manga_tag1 == pref_manga_tag2 {
            pref_manga_tag_text = format!(
                "{}{}{}{}{}",
                user_name1,
                &localised_text.same_tag_connector_manga,
                user_name2,
                &localised_text.same_tag_prefer_manga,
                pref_manga_tag1
            );
        } else {
            pref_manga_tag_text = format!(
                "{}{}{}{}{}{}{}",
                user_name1,
                &localised_text.diff_pref_tag_1_manga,
                pref_manga_tag1,
                &localised_text.diff_pref_tag_while_manga,
                user_name2,
                &localised_text.diff_pref_tag_2_manga,
                pref_manga_tag2
            );
        }

        let color = Colour::FABLED_PINK;
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
            println!("Cannot respond to slash command: {}", why);
        }
    } else {
        return "Language not found".to_string();
    }
    return "good".to_string();
}
