use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use serde_json::json;
use serenity::client::Context;
use serenity::model::application::interaction::application_command::CommandDataOptionValue;
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::prelude::interaction::application_command::{
    ApplicationCommandInteraction, CommandDataOption,
};
use serenity::model::Timestamp;
use serenity::utils::Colour;

use crate::cmd::anilist_module::get_nsfw_channel::get_nsfw;
use crate::cmd::anilist_module::struct_media::*;
use crate::cmd::general_module::get_guild_langage::get_guild_langage;
use crate::cmd::general_module::html_parser::convert_to_markdown;
use crate::cmd::general_module::lang_struct::MediaLocalisedText;
use crate::cmd::general_module::request::make_request_anilist;
use crate::cmd::general_module::trim::trim;

pub async fn embed(
    options: &[CommandDataOption],
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    query_id: &str,
    query_string: &str,
) -> String {
    let option = options
        .get(0)
        .expect("Expected name option")
        .resolved
        .as_ref()
        .expect("Expected name object");
    if let CommandDataOptionValue::String(value) = option {
        let query;
        if match value.parse::<i32>() {
            Ok(_) => true,
            Err(_) => false,
        } {
            query = query_id
        } else {
            query = query_string
        }

        let mut file = File::open("lang_file/anilist/media.json").expect("Failed to open file");
        let mut json = String::new();
        file.read_to_string(&mut json).expect("Failed to read file");

        let json_data: HashMap<String, MediaLocalisedText> =
            serde_json::from_str(&json).expect("Failed to parse JSON");

        let guild_id = command.guild_id.unwrap().0.to_string().clone();
        let lang_choice = get_guild_langage(guild_id).await;

        if let Some(localised_text) = json_data.get(lang_choice.as_str()) {
            let json = json!({"query": query, "variables": {"search": value}});
            let resp = make_request_anilist(json, false).await;
            // Get json
            let data: MediaWrapper = serde_json::from_str(&resp).unwrap();

            let is_nsfw = get_nsfw(command, ctx).await;
            if data.data.media.is_adult && !is_nsfw {
                return "not an NSFW channel".to_string();
            }

            let banner_image = format!("https://img.anili.st/media/{}", data.data.media.id);
            let mut desc = data
                .data
                .media
                .description
                .unwrap_or_else(|| "NA".to_string());
            let en_name = data
                .data
                .media
                .title
                .english
                .unwrap_or_else(|| "NA".to_string());
            let rj_name = data
                .data
                .media
                .title
                .romaji
                .unwrap_or_else(|| "NA".to_string());
            let thumbnail = data.data.media.cover_image.extra_large.unwrap_or_else(|| "https://imgs.search.brave.com/CYnhSvdQcm9aZe3wG84YY0B19zT2wlAuAkiAGu0mcLc/rs:fit:640:400:1/g:ce/aHR0cDovL3d3dy5m/cmVtb250Z3VyZHdh/cmEub3JnL3dwLWNv/bnRlbnQvdXBsb2Fk/cy8yMDIwLzA2L25v/LWltYWdlLWljb24t/Mi5wbmc".to_string());
            let site_url = data
                .data
                .media
                .site_url
                .unwrap_or_else(|| "https://example.com".to_string());
            let name = format!("{} / {}", en_name, rj_name);
            let format = data.data.media.format.unwrap_or_else(|| "N/A".to_string());
            let source = data.data.media.source.unwrap_or_else(|| "N/A".to_string());

            let start_y = data.data.media.start_date.year.unwrap_or_else(|| 0);
            let start_d = data.data.media.start_date.day.unwrap_or_else(|| 0);
            let start_m = data.data.media.start_date.month.unwrap_or_else(|| 0);
            let start_date = if start_y == 0 && start_d == 0 && start_m == 0 {
                "N/A".to_string()
            } else {
                format!("{}/{}/{}", start_d, start_m, start_y)
            };
            let end_y = data.data.media.end_date.year.unwrap_or_else(|| 0);
            let end_d = data.data.media.end_date.day.unwrap_or_else(|| 0);
            let end_m = data.data.media.end_date.month.unwrap_or_else(|| 0);
            let end_date = if end_y == 0 && end_d == 0 && end_m == 0 {
                "N/A".to_string()
            } else {
                format!("{}/{}/{}", start_d, start_m, start_y)
            };

            let mut staff = "".to_string();
            let staffs = data.data.media.staff.edges;
            for s in staffs {
                let full = s.node.name.full.unwrap_or_else(|| "N/A".to_string());
                let user = s
                    .node
                    .name
                    .user_preferred
                    .unwrap_or_else(|| "N/A".to_string());
                let role = s.role.unwrap_or_else(|| "N/A".to_string());
                staff.push_str(&format!(
                    "{}{}{}{}{}{}\n",
                    &localised_text.full_name,
                    full,
                    &localised_text.user_pref,
                    user,
                    &localised_text.role,
                    role
                ));
            }

            let info = format!(
                "{}{}{}{}{}{}{}{} \n {}",
                &localised_text.format,
                format,
                &localised_text.source,
                source,
                &localised_text.start_date,
                start_date,
                &localised_text.end_date,
                end_date,
                staff
            );
            let mut genre = "".to_string();
            let genre_list = data.data.media.genres;
            for g in genre_list {
                genre += &g.unwrap_or_else(|| "N/A".to_string());
                genre += "\n"
            }
            let mut tag = "".to_string();
            let tag_list = data.data.media.tags;
            for t in tag_list.iter().take(10) {
                let tag_name: String = t.name.as_ref().map_or("N/A".to_string(), |s| s.to_string());
                tag += &tag_name;
                tag += "\n";
            }

            let color = Colour::FABLED_PINK;

            desc = convert_to_markdown(desc);
            let lenght_diff = 4096 - desc.len() as i32;
            if lenght_diff <= 0 {
                desc = trim(desc, lenght_diff)
            }

            if let Err(why) = command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| {
                            message.embed(|m| {
                                m.title(name)
                                    .url(site_url)
                                    .timestamp(Timestamp::now())
                                    .color(color)
                                    .description(desc)
                                    .thumbnail(thumbnail)
                                    .image(banner_image)
                                    .field("Info", info, false)
                                    .fields(vec![("Genre", genre, true), ("Tag", tag, true)])
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
