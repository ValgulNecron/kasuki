use std::u32;

use regex::Regex;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use serenity::builder::CreateApplicationCommand;
use serenity::client::Context;
use serenity::model::application::interaction::application_command::CommandDataOptionValue;
use serenity::model::application::interaction::InteractionResponseType;
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
    Media: Media,
}

#[derive(Debug, Deserialize)]
struct Media {
    id: i64,
    description: Option<String>,
    title: Title,
    r#type: Option<String>,
    format: Option<String>,
    source: Option<String>,
    isAdult: bool,
    startDate: StartEndDate,
    endDate: StartEndDate,
    chapters: Option<i32>,
    volumes: Option<i32>,
    status: Option<String>,
    season: Option<String>,
    isLicensed: bool,
    coverImage: CoverImage,
    bannerImage: Option<String>,
    genres: Vec<Option<String>>,
    tags: Vec<Tag>,
    averageScore: Option<i32>,
    meanScore: Option<i32>,
    popularity: Option<i32>,
    favourites: Option<i32>,
    siteUrl: Option<String>,
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
    extraLarge: Option<String>,
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
    userPreferred: Option<String>,
}

const QUERY: &str = "
    query ($search: String) {
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
    staff {
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
        let banner_image = data.data.Media.bannerImage.unwrap_or_else(|| "https://imgs.search.brave.com/CYnhSvdQcm9aZe3wG84YY0B19zT2wlAuAkiAGu0mcLc/rs:fit:640:400:1/g:ce/aHR0cDovL3d3dy5m/cmVtb250Z3VyZHdh/cmEub3JnL3dwLWNv/bnRlbnQvdXBsb2Fk/cy8yMDIwLzA2L25v/LWltYWdlLWljb24t/Mi5wbmc".to_string());
        let desc_no_br = data.data.Media.description.unwrap_or_else(|| "NA".to_string()).replace("<br>", "");
        let re = Regex::new("<i>(.|\\n)*?</i>").unwrap();
        let desc = re.replace_all(&desc_no_br, "");
        let en_name = data.data.Media.title.english.unwrap_or_else(|| "NA".to_string());
        let rj_name = data.data.Media.title.romaji.unwrap_or_else(|| "NA".to_string());
        let thumbnail = data.data.Media.coverImage.extraLarge.unwrap_or_else(|| "https://imgs.search.brave.com/CYnhSvdQcm9aZe3wG84YY0B19zT2wlAuAkiAGu0mcLc/rs:fit:640:400:1/g:ce/aHR0cDovL3d3dy5m/cmVtb250Z3VyZHdh/cmEub3JnL3dwLWNv/bnRlbnQvdXBsb2Fk/cy8yMDIwLzA2L25v/LWltYWdlLWljb24t/Mi5wbmc".to_string());
        let site_url = data.data.Media.siteUrl.unwrap_or_else(|| "https://example.com".to_string());
        let name = format!("{} / {}", en_name, rj_name);
        let format = data.data.Media.format.unwrap_or_else(|| "N/A".to_string());
        let source = data.data.Media.source.unwrap_or_else(|| "N/A".to_string());

        let start_y = data.data.Media.startDate.year.unwrap_or_else(|| 0);
        let start_d = data.data.Media.startDate.day.unwrap_or_else(|| 0);
        let start_m = data.data.Media.startDate.month.unwrap_or_else(|| 0);
        let start_date = if start_y == 0 && start_d == 0 && start_m == 0 {
            "N/A".to_string()
        } else {
            format!("{}/{}/{}", start_d, start_m, start_y)
        };
        let end_y = data.data.Media.endDate.year.unwrap_or_else(|| 0);
        let end_d = data.data.Media.endDate.day.unwrap_or_else(|| 0);
        let end_m = data.data.Media.endDate.month.unwrap_or_else(|| 0);
        let end_date = if end_y == 0 && end_d == 0 && end_m == 0 {
            "N/A".to_string()
        } else {
            format!("{}/{}/{}", start_d, start_m, start_y)
        };

        let mut staff = "".to_string();
        let staffs = data.data.Media.staff.edges;
        for s in staffs {
            let full = s.node.name.full.unwrap_or_else(|| "N/A".to_string());
            let user = s.node.name.userPreferred.unwrap_or_else(|| "N/A".to_string());
            let role = s.role.unwrap_or_else(|| "N/A".to_string());
            staff.push_str(&format!("Full name: {} / User preferred: {} / Role: {}\n", full, user, role));
        }

        let info = format!("format : {} / source : {}\n start date : {} \n end date : {} \n {}", format, source, start_date, end_date, staff);
        let mut genre = "".to_string();
        let genre_list = data.data.Media.genres;
        for g in genre_list.iter().take(5) {
            genre += &g.unwrap_or_else(|| "N/A".to_string());
            genre += "\n"
        }
        let mut tag = "".to_string();
        let tag_list = data.data.Media.tags;
        for t in tag_list.iter().take(5) {
            let tag_name: String = t.name.as_ref().map_or("N/A".to_string(), |s| s.to_string());
            tag += &tag_name;
            tag += "\n";
        }

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
    command.name("anime").description("Info of an anime").create_option(
        |option| {
            option
                .name("animename")
                .description("Name of the anime you want to check")
                .kind(CommandOptionType::String)
                .required(true)
        },
    )
}