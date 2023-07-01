use std::u32;

use regex::Regex;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use serenity::builder::CreateApplicationCommand;
use serenity::client::Context;
use serenity::model::application::interaction::application_command::CommandDataOptionValue;
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::prelude::ChannelId;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::interaction::application_command::{ApplicationCommandInteraction, CommandDataOption};
use serenity::model::Timestamp;
use serenity::utils::Colour;

#[derive(Debug, Deserialize)]
struct Data {
    data: MediaData,
}

#[derive(Debug, Deserialize)]
struct MediaData {
    #[serde(rename = "Media")]
    media: Media,
}

#[derive(Debug, Deserialize)]
struct Media {
    id: i64,
    description: Option<String>,
    title: Title,
    r#type: Option<String>,
    format: Option<String>,
    source: Option<String>,
    #[serde(rename = "isAdult")]
    is_adult: bool,
    #[serde(rename = "startDate")]
    start_date: StartEndDate,
    #[serde(rename = "endDate")]
    end_date: StartEndDate,
    chapters: Option<i32>,
    volumes: Option<i32>,
    status: Option<String>,
    season: Option<String>,
    #[serde(rename = "isLicensed")]
    is_licensed: bool,
    #[serde(rename = "coverImage")]
    cover_image: CoverImage,
    #[serde(rename = "bannerImage")]
    banner_image: Option<String>,
    genres: Vec<Option<String>>,
    tags: Vec<Tag>,
    #[serde(rename = "averageScore")]
    average_score: Option<i32>,
    #[serde(rename = "meanScore")]
    mean_score: Option<i32>,
    popularity: Option<i32>,
    favourites: Option<i32>,
    #[serde(rename = "siteUrl")]
    site_url: Option<String>,
    staff: Staff,
}

#[derive(Debug, Deserialize)]
struct Title {
    romaji: Option<String>,
    english: Option<String>,
}

#[derive(Debug, Deserialize)]
struct StartEndDate {
    year: Option<i32>,
    month: Option<i32>,
    day: Option<i32>,
}

#[derive(Debug, Deserialize)]
struct CoverImage {
    #[serde(rename = "extraLarge")]
    extra_large: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Tag {
    name: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Staff {
    edges: Vec<Edge>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Edge {
    node: Node,
    id: Option<u32>,
    role: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Node {
    id: Option<u32>,
    name: Name,
}

#[derive(Debug, Deserialize, Serialize)]
struct Name {
    full: Option<String>,
    #[serde(rename = "userPreferred")]
    user_preferred: Option<String>,
}

const QUERY: &str = "
    query ($search: String, $limit: Int = 4) {
		Media (search: $search, type: MANGA, format: MANGA){
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

pub async fn run(options: &[CommandDataOption], ctx: &Context, command: &ApplicationCommandInteraction) -> String {
    let option = options
        .get(0)
        .expect("Expected name option")
        .resolved
        .as_ref()
        .expect("Expected name object");
    if let CommandDataOptionValue::String(name) = option {
        let client = Client::new();
        let json = json!({"query": QUERY, "variables": {"search": name}});
        let resp = client.post("https://graphql.anilist.co/")
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .body(json.to_string())
            .send()
            .await
            .unwrap()
            .text()
            .await;
        // Get json
        let data: Data = serde_json::from_str(&resp.unwrap()).unwrap();
        let hex_code = "#0D966D";
        let color_code = u32::from_str_radix(&hex_code[1..], 16).unwrap();
        let color = Colour::new(color_code);
        let banner_image_old = data.data.media.banner_image.unwrap_or_else(|| "https://imgs.search.brave.com/CYnhSvdQcm9aZe3wG84YY0B19zT2wlAuAkiAGu0mcLc/rs:fit:640:400:1/g:ce/aHR0cDovL3d3dy5m/cmVtb250Z3VyZHdh/cmEub3JnL3dwLWNv/bnRlbnQvdXBsb2Fk/cy8yMDIwLzA2L25v/LWltYWdlLWljb24t/Mi5wbmc".to_string());
        let banner_image = format!("https://img.anili.st/media/{}", data.data.media.id);
        let desc_no_br = data.data.media.description.unwrap_or_else(|| "NA".to_string()).replace("<br>", "");
        let re = Regex::new("<i>(.|\\n)*?</i>").unwrap();
        let desc = re.replace_all(&desc_no_br, "");
        let en_name = data.data.media.title.english.unwrap_or_else(|| "NA".to_string());
        let rj_name = data.data.media.title.romaji.unwrap_or_else(|| "NA".to_string());
        let thumbnail = data.data.media.cover_image.extra_large.unwrap_or_else(|| "https://imgs.search.brave.com/CYnhSvdQcm9aZe3wG84YY0B19zT2wlAuAkiAGu0mcLc/rs:fit:640:400:1/g:ce/aHR0cDovL3d3dy5m/cmVtb250Z3VyZHdh/cmEub3JnL3dwLWNv/bnRlbnQvdXBsb2Fk/cy8yMDIwLzA2L25v/LWltYWdlLWljb24t/Mi5wbmc".to_string());
        let site_url = data.data.media.site_url.unwrap_or_else(|| "https://example.com".to_string());
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
            let user = s.node.name.user_preferred.unwrap_or_else(|| "N/A".to_string());
            let role = s.role.unwrap_or_else(|| "N/A".to_string());
            staff.push_str(&format!("Full name: {} / User preferred: {} / Role: {}\n", full, user, role));
        }

        let info = format!("format : {} / source : {}\n start date : {} \n end date : {} \n {}", format, source, start_date, end_date, staff);
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

        if let Err(why) = command
            .create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| message.embed(
                        |m| {
                            m.title(name)
                                .url(site_url)
                                .timestamp(Timestamp::now())
                                .color(color)
                                .description(desc)
                                .thumbnail(thumbnail)
                                .image(banner_image)
                                .field("Info", info, false)
                                .fields(vec![
                                    ("Genre", genre, true),
                                    ("Tag", tag, true),
                                ])
                                .color(color)
                        })
                    )
            })
            .await
        {
            println!("Cannot respond to slash command: {}", why);
        }
    }
    return "good".to_string();
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("manga").description("Info of a manga").create_option(
        |option| {
            option
                .name("manganame")
                .description("Name of the manga you want to check")
                .kind(CommandOptionType::String)
                .required(true)
        },
    )
}