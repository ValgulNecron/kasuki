use cynic::{GraphQlResponse, QueryBuilder};
use serenity::all::{
    CommandInteraction, Context, CreateEmbed, CreateInteractionResponse,
    CreateInteractionResponseMessage, Timestamp,
};
use std::fmt::Display;

use crate::constant::{COLOR, UNKNOWN};
use crate::helper::convert_flavored_markdown::convert_anilist_flavored_to_discord_flavored_markdown;
use crate::helper::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use crate::helper::general_channel_info::get_nsfw;
use crate::helper::make_graphql_cached::make_request_anilist;
use crate::helper::trimer::trim;
use crate::structure::message::anilist_user::media::{load_localization_media, MediaLocalised};
#[cynic::schema("anilist")]
mod schema {}
#[derive(cynic::QueryVariables, Debug, Clone)]
pub struct MediaQuerryVariables<'a> {
    pub format_in: Option<Vec<Option<MediaFormat>>>,
    pub id: Option<i32>,
    pub media_type: Option<MediaType>,
    pub search: Option<&'a str>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Query", variables = "MediaQuerryVariables")]
pub struct MediaQuerry {
    #[arguments(search: $search, type: $media_type, id: $id, format_in: $format_in)]
    #[cynic(rename = "Media")]
    pub media: Option<Media>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct Media {
    pub id: i32,
    pub cover_image: Option<MediaCoverImage>,
    pub title: Option<MediaTitle>,
    pub volumes: Option<i32>,
    pub updated_at: Option<i32>,
    #[cynic(rename = "type")]
    pub type_: Option<MediaType>,
    pub trending: Option<i32>,
    pub synonyms: Option<Vec<Option<String>>>,
    pub tags: Option<Vec<Option<MediaTag>>>,
    pub status: Option<MediaStatus>,
    pub source: Option<MediaSource>,
    pub site_url: Option<String>,
    pub season_year: Option<i32>,
    pub season_int: Option<i32>,
    pub season: Option<MediaSeason>,
    pub popularity: Option<i32>,
    pub mod_notes: Option<String>,
    pub mean_score: Option<i32>,
    pub is_licensed: Option<bool>,
    pub is_adult: Option<bool>,
    pub hashtag: Option<String>,
    pub genres: Option<Vec<Option<String>>>,
    pub favourites: Option<i32>,
    pub format: Option<MediaFormat>,
    pub episodes: Option<i32>,
    pub end_date: Option<FuzzyDate>,
    pub duration: Option<i32>,
    pub description: Option<String>,
    pub country_of_origin: Option<CountryCode>,
    pub chapters: Option<i32>,
    pub banner_image: Option<String>,
    pub average_score: Option<i32>,
    pub auto_create_forum_thread: Option<bool>,
    pub characters: Option<CharacterConnection>,
    pub staff: Option<StaffConnection>,
    pub start_date: Option<FuzzyDate>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct StaffConnection {
    pub edges: Option<Vec<Option<StaffEdge>>>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct StaffEdge {
    pub role: Option<String>,
    pub node: Option<Staff>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct Staff {
    pub id: i32,
    pub name: Option<StaffName>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct StaffName {
    pub user_preferred: Option<String>,
    pub native: Option<String>,
    pub full: Option<String>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct MediaTag {
    pub category: Option<String>,
    pub description: Option<String>,
    pub id: i32,
    pub is_adult: Option<bool>,
    pub is_general_spoiler: Option<bool>,
    pub is_media_spoiler: Option<bool>,
    pub name: String,
    pub rank: Option<i32>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct MediaTitle {
    pub english: Option<String>,
    pub native: Option<String>,
    pub romaji: Option<String>,
    pub user_preferred: Option<String>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct MediaCoverImage {
    pub extra_large: Option<String>,
    pub medium: Option<String>,
    pub large: Option<String>,
    pub color: Option<String>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct FuzzyDate {
    pub year: Option<i32>,
    pub month: Option<i32>,
    pub day: Option<i32>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct CharacterConnection {
    pub edges: Option<Vec<Option<CharacterEdge>>>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct CharacterEdge {
    pub role: Option<CharacterRole>,
    pub node: Option<Character>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct Character {
    pub id: i32,
    pub name: Option<CharacterName>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct CharacterName {
    pub user_preferred: Option<String>,
    pub native: Option<String>,
    pub full: Option<String>,
}

#[derive(cynic::Enum, Clone, Copy, Debug)]
pub enum CharacterRole {
    Main,
    Supporting,
    Background,
}

#[derive(cynic::Enum, Clone, Copy, Debug)]
pub enum MediaFormat {
    Tv,
    TvShort,
    Movie,
    Special,
    Ova,
    Ona,
    Music,
    Manga,
    Novel,
    OneShot,
}

#[derive(cynic::Enum, Clone, Copy, Debug)]
pub enum MediaSeason {
    Winter,
    Spring,
    Summer,
    Fall,
}

#[derive(cynic::Enum, Clone, Copy, Debug)]
pub enum MediaSource {
    Original,
    Manga,
    LightNovel,
    VisualNovel,
    VideoGame,
    Other,
    Novel,
    Doujinshi,
    Anime,
    WebNovel,
    LiveAction,
    Game,
    Comic,
    MultimediaProject,
    PictureBook,
}

#[derive(cynic::Enum, Clone, Copy, Debug)]
pub enum MediaStatus {
    Finished,
    Releasing,
    NotYetReleased,
    Cancelled,
    Hiatus,
}

#[derive(cynic::Enum, Clone, Copy, Debug)]
pub enum MediaType {
    Anime,
    Manga,
}

#[derive(cynic::Scalar, Debug, Clone)]
pub struct CountryCode(pub String);

impl Display for CountryCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.clone())
    }
}

impl Display for MediaType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MediaType::Anime => write!(f, "Anime"),
            MediaType::Manga => write!(f, "Manga"),
        }
    }
}

impl Display for MediaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MediaStatus::Finished => write!(f, "Finished"),
            MediaStatus::Releasing => write!(f, "Releasing"),
            MediaStatus::NotYetReleased => write!(f, "Not Yet Released"),
            MediaStatus::Cancelled => write!(f, "Cancelled"),
            MediaStatus::Hiatus => write!(f, "Hiatus"),
        }
    }
}

impl Display for MediaSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MediaSource::Original => write!(f, "Original"),
            MediaSource::Manga => write!(f, "Manga"),
            MediaSource::LightNovel => write!(f, "Light Novel"),
            MediaSource::VisualNovel => write!(f, "Visual Novel"),
            MediaSource::VideoGame => write!(f, "Video Game"),
            MediaSource::Other => write!(f, "Other"),
            MediaSource::Novel => write!(f, "Novel"),
            MediaSource::Doujinshi => write!(f, "Doujinshi"),
            MediaSource::Anime => write!(f, "Anime"),
            MediaSource::WebNovel => write!(f, "Web Novel"),
            MediaSource::LiveAction => write!(f, "Live Action"),
            MediaSource::Game => write!(f, "Game"),
            MediaSource::Comic => write!(f, "Comic"),
            MediaSource::MultimediaProject => write!(f, "Multimedia Project"),
            MediaSource::PictureBook => write!(f, "Picture Book"),
        }
    }
}

impl Display for MediaFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MediaFormat::Tv => write!(f, "TV"),
            MediaFormat::TvShort => write!(f, "TV Short"),
            MediaFormat::Movie => write!(f, "Movie"),
            MediaFormat::Special => write!(f, "Special"),
            MediaFormat::Ova => write!(f, "OVA"),
            MediaFormat::Ona => write!(f, "ONA"),
            MediaFormat::Music => write!(f, "Music"),
            MediaFormat::Manga => write!(f, "Manga"),
            MediaFormat::Novel => write!(f, "Novel"),
            MediaFormat::OneShot => write!(f, "One Shot"),
        }
    }
}

impl Display for MediaSeason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MediaSeason::Winter => write!(f, "Winter"),
            MediaSeason::Spring => write!(f, "Spring"),
            MediaSeason::Summer => write!(f, "Summer"),
            MediaSeason::Fall => write!(f, "Fall"),
        }
    }
}

impl Display for CharacterRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CharacterRole::Main => write!(f, "Main"),
            CharacterRole::Supporting => write!(f, "Supporting"),
            CharacterRole::Background => write!(f, "Background"),
        }
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
fn embed_title(title: &MediaTitle) -> String {
    let en = title.english.clone();
    let rj = title.romaji.clone();
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
fn embed_desc(media: &Media) -> String {
    let mut desc = media.description.clone().unwrap_or_default();
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
fn get_genre(genres: &Vec<Option<String>>) -> String {
    genres
        .iter()
        .map(|string| string.clone().unwrap_or_default())
        .take(5)
        .collect::<Vec<String>>()
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
fn get_tag(tags: &Vec<Option<MediaTag>>) -> String {
    tags.iter()
        .map(|media_tag| media_tag.clone().unwrap().name)
        .take(5)
        .collect::<Vec<String>>()
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
fn get_url(media: &Media) -> String {
    media
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
fn get_thumbnail(media: &Media) -> String {
    match &media.clone().cover_image {
       Some(image) => {
           image.extra_large.clone().unwrap_or("https://imgs.search.brave.com/CYnhSvdQcm9aZe3wG84YY0B19zT2wlAuAkiAGu0mcLc/rs:fit:640:400:1/g:ce/aHR0cDovL3d3dy5m/cmVtb250Z3VyZHdh/cmEub3JnL3dwLWNv/\
    bnRlbnQvdXBsb2Fk/cy8yMDIwLzA2L25v/LWltYWdlLWljb24t/Mi5wbmc".to_string())
       }
       None => "https://imgs.search.brave.com/CYnhSvdQcm9aZe3wG84YY0B19zT2wlAuAkiAGu0mcLc/rs:fit:640:400:1/g:ce/aHR0cDovL3d3dy5m/cmVtb250Z3VyZHdh/cmEub3JnL3dwLWNv/\
           bnRlbnQvdXBsb2Fk/cy8yMDIwLzA2L25v/LWltYWdlLWljb24t/Mi5wbmc".to_string()
   }
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
pub fn get_banner(media: &Media) -> String {
    format!("https://img.anili.st/media/{}", media.id)
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
fn media_info(media: &Media, media_localised: &MediaLocalised) -> String {
    let mut desc = format!(
        "{} \n\n\
    {}",
        embed_desc(media),
        get_info(media, media_localised)
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
fn get_info(media: &Media, media_localised: &MediaLocalised) -> String {
    let text = media_localised.desc.clone();
    let format = match media.format {
        Some(f) => f.to_string(),
        None => UNKNOWN.to_string(),
    };
    let source = match media.source {
        Some(s) => s.to_string(),
        None => UNKNOWN.to_string(),
    };
    let start_data = match media.clone().start_date {
        Some(d) => get_date(&d),
        None => UNKNOWN.to_string(),
    };
    let end_data = match media.clone().end_date {
        Some(d) => get_date(&d),
        None => UNKNOWN.to_string(),
    };
    let staff_edges = media.staff.clone().unwrap().edges.unwrap_or_default();
    text.replace("$format$", format.as_str())
        .replace("$source$", source.as_str())
        .replace("$start_date$", start_data.as_str())
        .replace("$end_date$", end_data.as_str())
        .replace(
            "$staff_list$",
            get_staff(staff_edges, &media_localised.staff_text).as_str(),
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
fn get_date(date: &FuzzyDate) -> String {
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
fn get_staff(staff: Vec<Option<StaffEdge>>, staff_string: &str) -> String {
    let mut staff_text = String::new();
    for s in staff {
        let s = s.unwrap();
        let node = s.node.clone().unwrap();
        let text = staff_string;
        let name = node.name.unwrap();
        let full = name.full;
        let user_pref = name.user_preferred;
        let native = name.native;
        let staff_name = user_pref.unwrap_or(full.unwrap_or(native.unwrap_or(UNKNOWN.to_string())));
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
    data: Media,
) -> Result<(), AppError> {
    let is_adult = data.is_adult.unwrap_or(true);
    if is_adult && !get_nsfw(command_interaction, ctx).await {
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

    let title = data.title.clone().unwrap();
    let genres = data.genres.clone().unwrap_or_default();
    let tags = data.tags.clone().unwrap_or_default();
    let builder_embed = CreateEmbed::new()
        .timestamp(Timestamp::now())
        .color(COLOR)
        .description(media_info(&data, &media_localised))
        .title(embed_title(&title))
        .url(get_url(&data))
        .field(&media_localised.field1_title, get_genre(&genres), true)
        .field(&media_localised.field2_title, get_tag(&tags), true)
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

pub async fn get_media<'a>(
    value: String,
    var: MediaQuerryVariables<'a>,
) -> Result<Media, AppError> {
    let operation = MediaQuerry::build(var);
    let data: GraphQlResponse<MediaQuerry> = match make_request_anilist(operation, false).await
    {
        Ok(data) => match data.json::<GraphQlResponse<MediaQuerry>>().await {
            Ok(data) => data,
            Err(e) => {
                tracing::error!(?e);
                return Err(AppError {
                    message: format!("Error retrieving media with value {}\n{}", value, e),
                    error_type: ErrorType::WebRequest,
                    error_response_type: ErrorResponseType::Message,
                });
            }
        },
        Err(e) => {
            tracing::error!(?e);
            return Err(AppError {
                message: format!("Error retrieving media with value {}\n{}", value, e),
                error_type: ErrorType::WebRequest,
                error_response_type: ErrorResponseType::Message,
            });
        }
    };
    Ok(data.data.unwrap().media.unwrap())
}
