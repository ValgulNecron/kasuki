use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use regex::Regex;
use serde::Serialize;
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

use crate::cmd::anilist_module::struct_anime_autocomplete::Root;
use crate::cmd::anilist_module::struct_media::*;
use crate::cmd::general_module::get_guild_langage::get_guild_langage;
use crate::cmd::general_module::lang_struct::AnimeLocalisedText;
use crate::cmd::general_module::request::make_request;

// Query made to the anilist api.
const QUERY: &str = "
    query ($search: String, $limit: Int = 5) {
		Media (search: $search, type: ANIME){
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
    // Get the content of the first option.
    let option = options
        .get(0)
        .expect("Expected name option")
        .resolved
        .as_ref()
        .expect("Expected name object");
    // Check if the option variable contain the correct value.
    if let CommandDataOptionValue::String(name) = option {
        let mut file = File::open("lang_file/anilist/anime.json").expect("Failed to open file");
        let mut json = String::new();
        file.read_to_string(&mut json).expect("Failed to read file");

        let json_data: HashMap<String, AnimeLocalisedText> =
            serde_json::from_str(&json).expect("Failed to parse JSON");

        let guild_id = command.guild_id.unwrap().0.to_string().clone();
        let lang_choice = get_guild_langage(guild_id).await;

        if let Some(localised_text) = json_data.get(lang_choice.as_str()) {
            let json = json!({"query": QUERY, "variables": {"search": name}});
            let resp = make_request(json).await;
            // Get json
            let data: MediaData = match serde_json::from_str(&resp) {
                Ok(data) => data,
                Err(error) => {
                    println!("Error: {}", error);
                    return "Unable to find this anime.".to_string();
                }
            };

            let banner_image = format!("https://img.anili.st/media/{}", data.data.media.id);
            let desc_no_br = data
                .data
                .media
                .description
                .unwrap_or_else(|| "NA".to_string())
                .replace("<br>", "");
            let re = Regex::new("<i>(.|\\n)*?</i>").unwrap();
            let desc = re.replace_all(&desc_no_br, "");
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
                    &localised_text.desc_part_5,
                    full,
                    localised_text.desc_part_6,
                    user,
                    localised_text.desc_part_7,
                    role
                ));
            }

            let info = format!(
                "{}{}{}{}{}{}{}{} \n {}",
                &localised_text.desc_part_1,
                format,
                &localised_text.desc_part_2,
                source,
                &localised_text.desc_part_3,
                start_date,
                &localised_text.desc_part_4,
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
            for t in tag_list.iter().take(5) {
                let tag_name: String = t.name.as_ref().map_or("N/A".to_string(), |s| s.to_string());
                tag += &tag_name;
                tag += "\n";
            }
            let color = Colour::FABLED_PINK;

            if let Err(why) = command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| {
                            message.embed(|m| {
                                m.title(name)
                                    .url(site_url)
                                    .timestamp(Timestamp::now())
                                    .description(desc)
                                    .thumbnail(thumbnail)
                                    .image(banner_image)
                                    .field(&localised_text.desc_title, info, false)
                                    .fields(vec![
                                        (&localised_text.fields_name_1, genre, true),
                                        (&localised_text.fields_name_2, tag, true),
                                    ])
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
        .name("anime")
        .description("Info of an anime")
        .create_option(|option| {
            option
                .name("anime_name")
                .description("Name of the anime you want to check")
                .kind(CommandOptionType::String)
                .required(true)
                .set_autocomplete(true)
        })
}


pub async fn autocomplete(ctx: Context, command: AutocompleteInteraction) {
    let search = &command.data.options.first().unwrap().value;
    if let Some(search) = search {
        let query_str = "query($search: String, $type: MediaType, $count: Int) {
          Page(perPage: $count) {
		    media(search: $search, type: $type) {
		      id
		      title {
		        romaji
		        english
		      }
			}
		  }
		}";
        let json = json!({"query": query_str, "variables": {
            "search": search,
            "type": "ANIME",
            "count": 8,
        }});

        let res = make_request(json).await;
        let data: Root = serde_json::from_str(&res).unwrap();

        if let Some(media) = data.data.page.media {
            let suggestions: Vec<AutocompleteOption> = media
                .iter()
                .filter_map(|item| {
                    if let Some(item) = item {
                        Some(AutocompleteOption {
                            name: match &item.title {
                                Some(title) => {
                                    let english = title.english.clone();
                                    let romaji = title.romaji.clone();
                                    String::from(english.unwrap_or(
                                        romaji),
                                    )
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
                    response.set_choices(choices)
                })
                .await;
        }
    }
}

#[derive(Serialize, Debug)]
struct AutocompleteOption {
    name: String,
    value: String,
}