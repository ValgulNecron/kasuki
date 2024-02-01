use serde::Deserialize;
use serde_json::json;
use serenity::all::{
    CommandInteraction, Context, CreateEmbed, CreateInteractionResponse,
    CreateInteractionResponseMessage, Timestamp,
};

use crate::common::anilist_to_discord_markdown::convert_anilist_flavored_to_discord_flavored_markdown;
use crate::common::get_nsfw::get_nsfw;
use crate::common::make_anilist_request::make_request_anilist;
use crate::common::trimer::trim;
use crate::constant::{COLOR, UNKNOWN};
use crate::error_enum::AppError;
use crate::error_enum::AppError::Error;
use crate::error_enum::Error::{CommandSendingError, MediaGettingError, NotNSFWError};
use crate::lang_struct::anilist::media::{load_localization_media, MediaLocalised};

#[derive(Debug, Deserialize, Clone)]
pub struct MediaWrapper {
    pub data: MediaData,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MediaData {
    #[serde(rename = "Media")]
    pub media: Media,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Media {
    pub id: i64,
    pub description: Option<String>,
    pub title: Title,
    pub r#type: Option<String>,
    pub format: Option<String>,
    pub source: Option<String>,
    #[serde(rename = "isAdult")]
    pub is_adult: bool,
    #[serde(rename = "startDate")]
    pub start_date: StartEndDate,
    #[serde(rename = "endDate")]
    pub end_date: StartEndDate,
    pub chapters: Option<i32>,
    pub volumes: Option<i32>,
    pub status: Option<String>,
    pub season: Option<String>,
    #[serde(rename = "isLicensed")]
    pub is_licensed: bool,
    #[serde(rename = "coverImage")]
    pub cover_image: CoverImage,
    #[serde(rename = "bannerImage")]
    pub banner_image: Option<String>,
    pub genres: Vec<Option<String>>,
    pub tags: Vec<Tag>,
    #[serde(rename = "averageScore")]
    pub average_score: Option<i32>,
    #[serde(rename = "meanScore")]
    pub mean_score: Option<i32>,
    pub popularity: Option<i32>,
    pub favourites: Option<i32>,
    #[serde(rename = "siteUrl")]
    pub site_url: Option<String>,
    pub staff: Staff,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Title {
    pub romaji: Option<String>,
    pub english: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct StartEndDate {
    pub year: Option<i32>,
    pub month: Option<i32>,
    pub day: Option<i32>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CoverImage {
    #[serde(rename = "extraLarge")]
    pub extra_large: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Tag {
    pub name: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Staff {
    pub edges: Vec<Edge>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Edge {
    pub node: Node,
    pub id: Option<u32>,
    pub role: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Node {
    pub id: Option<u32>,
    pub name: Name,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Name {
    pub full: Option<String>,
    #[serde(rename = "userPreferred")]
    pub user_preferred: Option<String>,
}

impl MediaWrapper {
    pub async fn new_anime_by_id(id: String) -> Result<MediaWrapper, AppError> {
        let query_id: &str = "
    query ($search: Int, $limit: Int = 5) {
		Media (id: $search, type: ANIME){
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

        let json = json!({"query": query_id, "variables": {"search": id}});
        let resp = make_request_anilist(json, false).await;
        // Get json
        serde_json::from_str(&resp).map_err(|e| {
            Error(MediaGettingError(format!(
                "Error getting the media with id {}. {}",
                id, e
            )))
        })
    }

    pub async fn new_anime_by_search(search: &String) -> Result<MediaWrapper, AppError> {
        let query_string: &str = "
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
        let json = json!({"query": query_string, "variables": {"search": search}});
        let resp = make_request_anilist(json, false).await;
        // Get json
        serde_json::from_str(&resp).map_err(|e| {
            Error(MediaGettingError(format!(
                "Error getting the media with name {}. {}",
                search, e
            )))
        })
    }

    pub async fn new_manga_by_id(id: String) -> Result<MediaWrapper, AppError> {
        let query_id: &str = "
    query ($search: Int, $limit: Int = 5, $format: MediaFormat = NOVEL) {
		Media (id: $search, type: MANGA, format_not: $format){
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

        let json = json!({"query": query_id, "variables": {"search": id}});
        let resp = make_request_anilist(json, false).await;
        // Get json
        serde_json::from_str(&resp).map_err(|e| {
            Error(MediaGettingError(format!(
                "Error getting the media with id {}. {}",
                id, e
            )))
        })
    }

    pub async fn new_manga_by_search(search: &String) -> Result<MediaWrapper, AppError> {
        let query_string: &str = "
    query ($search: String, $limit: Int = 5, $format: MediaFormat = NOVEL) {
		Media (search: $search, type: MANGA, format_not: $format){
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
        let json = json!({"query": query_string, "variables": {"search": search}});
        let resp = make_request_anilist(json, false).await;
        // Get json
        serde_json::from_str(&resp).map_err(|e| {
            Error(MediaGettingError(format!(
                "Error getting the media with name {}. {}",
                search, e
            )))
        })
    }

    pub async fn new_ln_by_id(id: String) -> Result<MediaWrapper, AppError> {
        let query_id: &str = "
    query ($search: Int, $limit: Int = 5, $format: MediaFormat = NOVEL) {
		Media (id: $search, type: MANGA, format: $format){
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

        let json = json!({"query": query_id, "variables": {"search": id}});
        let resp = make_request_anilist(json, false).await;
        // Get json
        serde_json::from_str(&resp).map_err(|e| {
            Error(MediaGettingError(format!(
                "Error getting the media with id {}. {}",
                id, e
            )))
        })
    }

    pub async fn new_ln_by_search(search: &String) -> Result<MediaWrapper, AppError> {
        let query_string: &str = "
    query ($search: String, $limit: Int = 5, $format: MediaFormat = NOVEL) {
		Media (search: $search, type: MANGA, format: $format){
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

        let json = json!({"query": query_string, "variables": {"search": search}});
        let resp = make_request_anilist(json, false).await;
        // Get json
        serde_json::from_str(&resp).map_err(|e| {
            Error(MediaGettingError(format!(
                "Error getting the media with name {}. {}",
                search, e
            )))
        })
    }
}

fn embed_title(data: &MediaWrapper) -> String {
    let en = data.data.media.title.english.clone();
    let rj = data.data.media.title.romaji.clone();
    let en = en.unwrap_or(String::from(""));
    let rj = rj.unwrap_or(String::from(""));
    let mut title = String::new();
    let mut total = 0;
    match en.as_str() {
        "" => {}
        _ => {
            total += 1;
            title.push_str(en.as_str())
        }
    }

    match rj.as_str() {
        "\"\"" => {}
        _ => {
            if total == 1 {
                title.push_str(" / ");
                title.push_str(rj.as_str())
            } else {
                title.push_str(rj.as_str())
            }
        }
    }

    title
}

fn embed_desc(data: &MediaWrapper) -> String {
    let mut desc = data.data.media.description.clone().unwrap_or_default();
    desc = convert_anilist_flavored_to_discord_flavored_markdown(desc);
    let lenght_diff = 4096 - desc.len() as i32;
    if lenght_diff <= 0 {
        desc = trim(desc, lenght_diff)
    }
    desc
}

fn get_genre(data: &MediaWrapper) -> String {
    data.data
        .media
        .genres
        .iter()
        .filter_map(|x| x.as_ref())
        .map(|s| s.as_str())
        .take(5)
        .collect::<Vec<&str>>()
        .join("\n")
}

fn get_tag(data: &MediaWrapper) -> String {
    data.data
        .media
        .tags
        .iter()
        .filter_map(|x| x.name.as_ref())
        .map(|s| s.as_str())
        .take(5)
        .collect::<Vec<&str>>()
        .join("\n")
}

fn get_url(data: &MediaWrapper) -> String {
    data.data
        .media
        .site_url
        .clone()
        .unwrap_or("https://example.com".to_string())
}

fn get_thumbnail(data: &MediaWrapper) -> String {
    data.data.media.cover_image.extra_large.clone().unwrap_or("https://imgs.search.brave.com/CYnhSvdQcm9aZe3wG84YY0B19zT2wlAuAkiAGu0mcLc/rs:fit:640:400:1/g:ce/aHR0cDovL3d3dy5m/cmVtb250Z3VyZHdh/cmEub3JnL3dwLWNv/bnRlbnQvdXBsb2Fk/cy8yMDIwLzA2L25v/LWltYWdlLWljb24t/Mi5wbmc".to_string())
}

pub fn get_banner(data: &MediaWrapper) -> String {
    format!("https://img.anili.st/media/{}", data.data.media.id)
}

fn media_info(data: &MediaWrapper, media_localised: &MediaLocalised) -> String {
    let mut desc = format!(
        "{} \n\n\
    {}",
        embed_desc(data),
        get_info(&data.data.media, media_localised)
    );
    desc = convert_anilist_flavored_to_discord_flavored_markdown(desc);
    let lenght_diff = 4096 - desc.len() as i32;
    if lenght_diff <= 0 {
        desc = trim(desc, lenght_diff)
    }
    desc
}

fn get_info(data: &Media, media_localised: &MediaLocalised) -> String {
    let text = media_localised.desc.clone();
    let format = data.format.clone();
    let source = data.source.clone();
    text.replace("$format$", format.unwrap_or(UNKNOWN.to_string()).as_str())
        .replace("$source$", source.unwrap_or(UNKNOWN.to_string()).as_str())
        .replace("$start_date$", get_date(&data.start_date).as_str())
        .replace("$end_date$", get_date(&data.end_date).as_str())
        .replace(
            "$staff_list$",
            get_staff(&data.staff.edges, &media_localised.staff_text).as_str(),
        )
}

fn get_date(date: &StartEndDate) -> String {
    let date_y = date.year.unwrap_or(0);
    let date_d = date.day.unwrap_or(0);
    let date_m = date.month.unwrap_or(0);
    if date_y == 0 && date_d == 0 && date_m == 0 {
        UNKNOWN.to_string()
    } else {
        format!("{}/{}/{}", date_d, date_m, date_y)
    }
}

fn get_staff(staff: &Vec<Edge>, staff_string: &str) -> String {
    let mut staff_text = String::new();
    for s in staff {
        let text = staff_string;
        let node = &s.node;
        let name = &node.name;
        let full = name.full.clone();
        let user_pref = name.user_preferred.clone();
        let staff_name = user_pref.unwrap_or(full.unwrap_or(UNKNOWN.to_string()));
        let s_role = s.role.clone();
        let role = s_role.unwrap_or(UNKNOWN.to_string());
        staff_text.push_str(
            text.replace("$name$", staff_name.as_str())
                .replace("$role$", role.as_str())
                .as_str(),
        )
    }

    staff_text
}

pub async fn send_embed(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    data: MediaWrapper,
) -> Result<(), AppError> {
    if data.data.media.is_adult && !get_nsfw(command_interaction, ctx).await {
        return Err(Error(NotNSFWError(String::from(
            "The channel is not nsfw.",
        ))));
    }

    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };

    let media_localised = load_localization_media(guild_id).await?;

    let builder_embed = CreateEmbed::new()
        .timestamp(Timestamp::now())
        .color(COLOR)
        .description(media_info(&data, &media_localised))
        .title(embed_title(&data))
        .url(get_url(&data))
        .field(&media_localised.field1_title, get_genre(&data), true)
        .field(&media_localised.field2_title, get_tag(&data), true)
        .thumbnail(get_thumbnail(&data))
        .image(get_banner(&data));

    let builder_message = CreateInteractionResponseMessage::new().embed(builder_embed);

    let builder = CreateInteractionResponse::Message(builder_message);

    command_interaction
        .create_response(&ctx.http, builder)
        .await
        .map_err(|e| {
            Error(CommandSendingError(format!(
                "Error while sending the command {}",
                e
            )))
        })
}
