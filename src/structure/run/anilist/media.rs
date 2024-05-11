use serde::Deserialize;
use serde_json::json;
use serenity::all::{
    CommandInteraction, Context, CreateEmbed, CreateInteractionResponse,
    CreateInteractionResponseMessage, Timestamp,
};

use crate::helper::convert_flavored_markdown::convert_anilist_flavored_to_discord_flavored_markdown;
use crate::helper::general_channel_info::get_nsfw;
use crate::helper::make_anilist_cached_request::make_request_anilist;
use crate::helper::trimer::trim;
use crate::constant::{COLOR, UNKNOWN};
use crate::helper::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use crate::structure::message::anilist_user::media::{load_localization_media, MediaLocalised};

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

/// `MediaWrapper` is an implementation block for the `MediaWrapper` struct.
impl MediaWrapper {
    /// `new_anime_by_id` is an asynchronous function that creates a new anime by ID.
    /// It takes an `id` as a parameter.
    /// `id` is a String that represents the ID of the anime.
    /// It returns a `Result` that contains a `MediaWrapper` or an `AppError`.
    ///
    /// This function first defines a GraphQL query string that takes an `id` as a variable.
    /// It then creates a JSON object with the query string and the variable.
    /// The `id` variable is set to the `id` parameter.
    /// It makes a request to AniList with the JSON object and waits for the response.
    /// It then deserializes the response into a `MediaWrapper` and returns it.
    ///
    /// # Arguments
    ///
    /// * `id` - A String that represents the ID of the anime.
    ///
    /// # Returns
    ///
    /// * `Result<MediaWrapper, AppError>` - A Result that contains a `MediaWrapper` or an `AppError`.
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
        serde_json::from_str(&resp).map_err(|e| AppError {
            message: format!("Error getting the media with id {}. {}", id, e),
            error_type: ErrorType::WebRequest,
            error_response_type: ErrorResponseType::Message,
        })
    }

    /// `new_anime_by_search` is an asynchronous function that creates a new anime by search.
    /// It takes a `search` as a parameter.
    /// `search` is a reference to a String that represents the search query.
    /// It returns a `Result` that contains a `MediaWrapper` or an `AppError`.
    ///
    /// This function first defines a GraphQL query string that takes a `search` as a variable.
    /// It then creates a JSON object with the query string and the variable.
    /// The `search` variable is set to the `search` parameter.
    /// It makes a request to AniList with the JSON object and waits for the response.
    /// It then deserializes the response into a `MediaWrapper` and returns it.
    ///
    /// # Arguments
    ///
    /// * `search` - A reference to a String that represents the search query.
    ///
    /// # Returns
    ///
    /// * `Result<MediaWrapper, AppError>` - A Result that contains a `MediaWrapper` or an `AppError`.
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
        serde_json::from_str(&resp).map_err(|e| AppError {
            message: format!("Error getting the media with name {}. {}", search, e),
            error_type: ErrorType::WebRequest,
            error_response_type: ErrorResponseType::Message,
        })
    }

    /// `new_manga_by_id` is an asynchronous function that creates a new manga by ID.
    /// It takes an `id` as a parameter.
    /// `id` is a String that represents the ID of the manga.
    /// It returns a `Result` that contains a `MediaWrapper` or an `AppError`.
    ///
    /// This function first defines a GraphQL query string that takes an `id` as a variable.
    /// It then creates a JSON object with the query string and the variable.
    /// The `id` variable is set to the `id` parameter.
    /// It makes a request to AniList with the JSON object and waits for the response.
    /// It then deserializes the response into a `MediaWrapper` and returns it.
    ///
    /// # Arguments
    ///
    /// * `id` - A String that represents the ID of the manga.
    ///
    /// # Returns
    ///
    /// * `Result<MediaWrapper, AppError>` - A Result that contains a `MediaWrapper` or an `AppError`.
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
        serde_json::from_str(&resp).map_err(|e| AppError {
            message: format!("Error getting the media with id {}. {}", id, e),
            error_type: ErrorType::WebRequest,
            error_response_type: ErrorResponseType::Message,
        })
    }

    /// `new_manga_by_search` is an asynchronous function that creates a new manga by search.
    /// It takes a `search` as a parameter.
    /// `search` is a reference to a String that represents the search query.
    /// It returns a `Result` that contains a `MediaWrapper` or an `AppError`.
    ///
    /// This function first defines a GraphQL query string that takes a `search` as a variable.
    /// It then creates a JSON object with the query string and the variable.
    /// The `search` variable is set to the `search` parameter.
    /// It makes a request to AniList with the JSON object and waits for the response.
    /// It then deserializes the response into a `MediaWrapper` and returns it.
    ///
    /// # Arguments
    ///
    /// * `search` - A reference to a String that represents the search query.
    ///
    /// # Returns
    ///
    /// * `Result<MediaWrapper, AppError>` - A Result that contains a `MediaWrapper` or an `AppError`.
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
        serde_json::from_str(&resp).map_err(|e| AppError {
            message: format!("Error getting the media with name {}. {}", search, e),
            error_type: ErrorType::WebRequest,
            error_response_type: ErrorResponseType::Message,
        })
    }

    /// `new_ln_by_id` is an asynchronous function that creates a new light novel by ID.
    /// It takes an `id` as a parameter.
    /// `id` is a String that represents the ID of the light novel.
    /// It returns a `Result` that contains a `MediaWrapper` or an `AppError`.
    ///
    /// This function first defines a GraphQL query string that takes an `id` as a variable.
    /// It then creates a JSON object with the query string and the variable.
    /// The `id` variable is set to the `id` parameter.
    /// It makes a request to AniList with the JSON object and waits for the response.
    /// It then deserializes the response into a `MediaWrapper` and returns it.
    ///
    /// # Arguments
    ///
    /// * `id` - A String that represents the ID of the light novel.
    ///
    /// # Returns
    ///
    /// * `Result<MediaWrapper, AppError>` - A Result that contains a `MediaWrapper` or an `AppError`.
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
        serde_json::from_str(&resp).map_err(|e| AppError {
            message: format!("Error getting the media with id {}. {}", id, e),
            error_type: ErrorType::WebRequest,
            error_response_type: ErrorResponseType::Message,
        })
    }

    /// `new_ln_by_search` is an asynchronous function that creates a new light novel by search.
    /// It takes a `search` as a parameter.
    /// `search` is a reference to a String that represents the search query.
    /// It returns a `Result` that contains a `MediaWrapper` or an `AppError`.
    ///
    /// This function first defines a GraphQL query string that takes a `search` as a variable.
    /// It then creates a JSON object with the query string and the variable.
    /// The `search` variable is set to the `search` parameter.
    /// It makes a request to AniList with the JSON object and waits for the response.
    /// It then deserializes the response into a `MediaWrapper` and returns it.
    ///
    /// # Arguments
    ///
    /// * `search` - A reference to a String that represents the search query.
    ///
    /// # Returns
    ///
    /// * `Result<MediaWrapper, AppError>` - A Result that contains a `MediaWrapper` or an `AppError`.
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
        serde_json::from_str(&resp).map_err(|e| AppError {
            message: format!("Error getting the media with name {}. {}", search, e),
            error_type: ErrorType::WebRequest,
            error_response_type: ErrorResponseType::Message,
        })
    }
}

/// `embed_title` is a function that creates a title for the embed.
/// It takes a `data` as a parameter.
/// `data` is a reference to a `MediaWrapper` that represents the media wrapper.
/// It returns a String that represents the title of the embed.
///
/// This function first gets the English and Romaji titles from the `data`.
/// It then checks if the English title is not empty and adds it to the title.
/// It also checks if the Romaji title is not empty and adds it to the title.
/// If the English title is not empty and the Romaji title is not empty, it separates them with a slash.
///
/// # Arguments
///
/// * `data` - A reference to a `MediaWrapper` that represents the media wrapper.
///
/// # Returns
///
/// * `String` - A String that represents the title of the embed.
fn embed_title(data: &MediaWrapper) -> String {
    let en = data.data.media.title.english.clone();
    let rj = data.data.media.title.romaji.clone();
    let en = en.unwrap_or_default();
    let rj = rj.unwrap_or_default();
    let mut title = String::new();
    let mut has_en_title = false;
    match en.as_str() {
        "" => {}
        _ => {
            has_en_title = true;
            title.push_str(en.as_str())
        }
    }

    match rj.as_str() {
        "" => {}
        _ => {
            if has_en_title {
                title.push_str(" / ");
                title.push_str(rj.as_str())
            } else {
                title.push_str(rj.as_str())
            }
        }
    }

    title
}

/// `embed_desc` is a function that creates a description for the embed.
/// It takes a `data` as a parameter.
/// `data` is a reference to a `MediaWrapper` that represents the media wrapper.
/// It returns a String that represents the description of the embed.
///
/// This function first gets the description from the `data`.
/// It then converts the AniList flavored markdown in the description to Discord flavored markdown.
/// It checks if the length of the description exceeds the limit.
/// If it does, it trims the description to fit the limit.
///
/// # Arguments
///
/// * `data` - A reference to a `MediaWrapper` that represents the media wrapper.
///
/// # Returns
///
/// * `String` - A String that represents the description of the embed.
fn embed_desc(data: &MediaWrapper) -> String {
    let mut desc = data.data.media.description.clone().unwrap_or_default();
    desc = convert_anilist_flavored_to_discord_flavored_markdown(desc);
    let length_diff = 4096 - desc.len() as i32;
    if length_diff <= 0 {
        desc = trim(desc, length_diff)
    }
    desc
}

/// `get_genre` is a function that gets the genres of the media.
/// It takes a `data` as a parameter.
/// `data` is a reference to a `MediaWrapper` that represents the media wrapper.
/// It returns a String that represents the genres of the media.
///
/// This function first gets the genres from the `data`.
/// It then filters the genres that are not None and converts them to a string.
/// It takes the first 5 genres and joins them with a newline.
///
/// # Arguments
///
/// * `data` - A reference to a `MediaWrapper` that represents the media wrapper.
///
/// # Returns
///
/// * `String` - A String that represents the genres of the media.
fn get_genre(data: &MediaWrapper) -> String {
    data.data
        .media
        .genres
        .iter()
        .filter_map(|genre| genre.as_ref())
        .map(|string| string.as_str())
        .take(5)
        .collect::<Vec<&str>>()
        .join("\n")
}

/// `get_tag` is a function that gets the tags of the media.
/// It takes a `data` as a parameter.
/// `data` is a reference to a `MediaWrapper` that represents the media wrapper.
/// It returns a String that represents the tags of the media.
///
/// This function first gets the tags from the `data`.
/// It then filters the tags that are not None and converts them to a string.
/// It takes the first 5 tags and joins them with a newline.
///
/// # Arguments
///
/// * `data` - A reference to a `MediaWrapper` that represents the media wrapper.
///
/// # Returns
///
/// * `String` - A String that represents the tags of the media.
fn get_tag(data: &MediaWrapper) -> String {
    data.data
        .media
        .tags
        .iter()
        .filter_map(|tag| tag.name.as_ref())
        .map(|string| string.as_str())
        .take(5)
        .collect::<Vec<&str>>()
        .join("\n")
}

/// `get_url` is a function that gets the URL of the media.
/// It takes a `data` as a parameter.
/// `data` is a reference to a `MediaWrapper` that represents the media wrapper.
/// It returns a String that represents the URL of the media.
///
/// This function first gets the URL from the `data`.
/// If the URL is None, it returns a default URL.
///
/// # Arguments
///
/// * `data` - A reference to a `MediaWrapper` that represents the media wrapper.
///
/// # Returns
///
/// * `String` - A String that represents the URL of the media.
fn get_url(data: &MediaWrapper) -> String {
    data.data
        .media
        .site_url
        .clone()
        .unwrap_or("https://example.com".to_string())
}

/// `get_thumbnail` is a function that gets the thumbnail of the media.
/// It takes a `data` as a parameter.
/// `data` is a reference to a `MediaWrapper` that represents the media wrapper.
/// It returns a String that represents the thumbnail of the media.
///
/// This function first gets the thumbnail from the `data`.
/// If the thumbnail is None, it returns a default thumbnail.
///
/// # Arguments
///
/// * `data` - A reference to a `MediaWrapper` that represents the media wrapper.
///
/// # Returns
///
/// * `String` - A String that represents the thumbnail of the media.
fn get_thumbnail(data: &MediaWrapper) -> String {
    data.data.media.cover_image.extra_large.clone().unwrap_or("https://imgs.search.brave.com/CYnhSvdQcm9aZe3wG84YY0B19zT2wlAuAkiAGu0mcLc/rs:fit:640:400:1/g:ce/aHR0cDovL3d3dy5m/cmVtb250Z3VyZHdh/cmEub3JnL3dwLWNv/\
    bnRlbnQvdXBsb2Fk/cy8yMDIwLzA2L25v/LWltYWdlLWljb24t/Mi5wbmc".to_string())
}

/// `get_banner` is a function that gets the banner of the media.
/// It takes a `data` as a parameter.
/// `data` is a reference to a `MediaWrapper` that represents the media wrapper.
/// It returns a String that represents the banner of the media.
///
/// This function first gets the ID of the media from the `data`.
/// It then formats the ID into a URL that represents the banner of the media.
///
/// # Arguments
///
/// * `data` - A reference to a `MediaWrapper` that represents the media wrapper.
///
/// # Returns
///
/// * `String` - A String that represents the banner of the media.
pub fn get_banner(data: &MediaWrapper) -> String {
    format!("https://img.anili.st/media/{}", data.data.media.id)
}

/// `media_info` is a function that gets the information of the media.
/// It takes `data` and `media_localised` as parameters.
/// `data` is a reference to a `MediaWrapper` that represents the media wrapper.
/// `media_localised` is a reference to a `MediaLocalised` that represents the localized media.
/// It returns a String that represents the information of the media.
///
/// This function first gets the description and the information of the media from the `data`.
/// It then converts the AniList flavored markdown in the description to Discord flavored markdown.
/// It checks if the length of the description exceeds the limit.
/// If it does, it trims the description to fit the limit.
///
/// # Arguments
///
/// * `data` - A reference to a `MediaWrapper` that represents the media wrapper.
/// * `media_localised` - A reference to a `MediaLocalised` that represents the localized media.
///
/// # Returns
///
/// * `String` - A String that represents the information of the media.
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

/// `get_info` is a function that gets the information of the media.
/// It takes `data` and `media_localised` as parameters.
/// `data` is a reference to a `Media` that represents the media.
/// `media_localised` is a reference to a `MediaLocalised` that represents the localized media.
/// It returns a String that represents the information of the media.
///
/// This function first gets the text, the format, and the source from the `media_localised`.
/// It then replaces the placeholders in the text with the format, the source, the start date, the end date, and the staff list.
///
/// # Arguments
///
/// * `data` - A reference to a `Media` that represents the media.
/// * `media_localised` - A reference to a `MediaLocalised` that represents the localized media.
///
/// # Returns
///
/// * `String` - A String that represents the information of the media.
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

/// `get_date` is a function that gets the date.
/// It takes a `date` as a parameter.
/// `date` is a reference to a `StartEndDate` that represents the start or end date.
/// It returns a String that represents the date.
///
/// This function first gets the year, the day, and the month from the `date`.
/// If the year, the day, and the month are all 0, it returns a default date.
/// Otherwise, it formats the year, the day, and the month into a date string.
///
/// # Arguments
///
/// * `date` - A reference to a `StartEndDate` that represents the start or end date.
///
/// # Returns
///
/// * `String` - A String that represents the date.
fn get_date(date: &StartEndDate) -> String {
    let date_y = date.year.unwrap_or(0);
    let date_d = date.day.unwrap_or(0);
    let date_m = date.month.unwrap_or(0);
    if date_y == 0 && date_d == 0 && date_m == 0 {
        UNKNOWN.to_string()
    } else {
        let mut date_of_birth_string = String::new();

        let mut has_month: bool = false;
        let mut has_day: bool = false;

        if let Some(m) = date.month {
            date_of_birth_string.push_str(format!("{:02}", m).as_str());
            has_month = true
        }

        if let Some(d) = date.day {
            if has_month {
                date_of_birth_string.push('/')
            }
            date_of_birth_string.push_str(format!("{:02}", d).as_str());
            has_day = true
        }

        if let Some(y) = date.year {
            if has_day {
                date_of_birth_string.push('/')
            }
            date_of_birth_string.push_str(format!("{:04}", y).as_str());
        }

        date_of_birth_string
    }
}

/// `get_staff` is a function that gets the staff of the media.
/// It takes `staff` and `staff_string` as parameters.
/// `staff` is a reference to a vector of `Edge` that represents the staff.
/// `staff_string` is a reference to a string that represents the staff string.
/// It returns a String that represents the staff of the media.
///
/// This function first creates a new string for the staff text.
/// It then iterates over the staff and gets the node and the name from each staff.
/// It gets the full name and the user preferred name from the name.
/// It sets the staff name to the user preferred name if it is not None, otherwise it sets it to the full name.
/// It gets the role from the staff and sets it to a default string if it is None.
/// It then replaces the placeholders in the staff string with the staff name and the role and adds it to the staff text.
///
/// # Arguments
///
/// * `staff` - A reference to a vector of `Edge` that represents the staff.
/// * `staff_string` - A reference to a string that represents the staff string.
///
/// # Returns
///
/// * `String` - A String that represents the staff of the media.
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

/// `send_embed` is an asynchronous function that sends an embed.
/// It takes `ctx`, `command_interaction`, and `data` as parameters.
/// `ctx` is a Context that represents the context.
/// `command_interaction` is a CommandInteraction that represents the command interaction.
/// `data` is a MediaWrapper that represents the media wrapper.
///
/// This function first checks if the media is adult and if the channel is not NSFW.
/// If it is, it returns an error.
/// It then gets the guild ID from the `command_interaction`.
/// It loads the localized media using the guild ID.
/// It creates a new embed with the description, the title, the URL, the genre, the tag, the thumbnail, and the image of the media.
/// It creates a new interaction response message with the embed.
/// It creates a new interaction response with the interaction response message.
/// It then sends the interaction response using the `command_interaction`.
///
/// # Arguments
///
/// * `ctx` - A Context that represents the context.
/// * `command_interaction` - A CommandInteraction that represents the command interaction.
/// * `data` - A MediaWrapper that represents the media wrapper.
///
/// # Returns
///
/// * `Result<(), AppError>` - A Result that represents the result of the function. It returns an empty Ok if the function is successful, otherwise it returns an Err with an AppError.
pub async fn send_embed(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    data: MediaWrapper,
) -> Result<(), AppError> {
    if data.data.media.is_adult && !get_nsfw(command_interaction, ctx).await {
        return Err(AppError {
            message: String::from("The channel is not nsfw but the media you requested is."),
            error_type: ErrorType::Command,
            error_response_type: ErrorResponseType::Message,
        });
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
        .map_err(|e| AppError {
            message: format!("Error sending the media embed. {}", e),
            error_type: ErrorType::Command,
            error_response_type: ErrorResponseType::Message,
        })
}
