use std::error::Error;
use std::fmt::Display;

use crate::config::DbConfig;
use crate::constant::{COLOR, UNKNOWN};
use crate::helper::convert_flavored_markdown::convert_anilist_flavored_to_discord_flavored_markdown;
use crate::helper::error_management::error_dispatch;
use crate::helper::general_channel_info::get_nsfw;
use crate::helper::trimer::trim;
use crate::structure::message::anilist_user::media::load_localization_media;
use serenity::all::{
    CommandInteraction, Context, CreateEmbed, CreateInteractionResponse,
    CreateInteractionResponseMessage, Timestamp,
};

#[cynic::schema("anilist")]

mod schema {}

#[derive(cynic::QueryVariables, Debug, Clone)]

pub struct MediaQuerryIdVariables {
    pub format_in: Option<Vec<Option<MediaFormat>>>,
    pub id: Option<i32>,
    pub media_type: Option<MediaType>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Query", variables = "MediaQuerryIdVariables")]

pub struct MediaQuerryId {
    #[arguments(type: $ media_type, id: $ id, format_in: $ format_in)]
    #[cynic(rename = "Media")]
    pub media: Option<Media>,
}

#[derive(cynic::QueryVariables, Debug, Clone)]

pub struct MediaQuerrySearchVariables<'a> {
    pub format_in: Option<Vec<Option<MediaFormat>>>,
    pub media_type: Option<MediaType>,
    pub search: Option<&'a str>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Query", variables = "MediaQuerrySearchVariables")]

pub struct MediaQuerrySearch {
    #[arguments(search: $ search, type: $ media_type, format_in: $ format_in)]
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

fn get_genre(genres: &[Option<String>]) -> String {

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

fn get_tag(tags: &[Option<MediaTag>]) -> String {

    tags.iter()
        .map(|media_tag| {

            media_tag
                .clone()
                .unwrap_or(MediaTag {
                    category: None,
                    description: None,
                    id: 0,
                    is_adult: None,
                    is_general_spoiler: None,
                    is_media_spoiler: None,
                    name: "".to_string(),
                    rank: None,
                })
                .name
        })
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

fn get_staff(staff: Vec<Option<StaffEdge>>) -> String {

    let mut staff_text = String::new();

    // iterate over staff with index
    let mut i = 0;

    for s in staff.into_iter() {

        if i > 4 {

            break;
        }

        let s = match s {
            Some(s) => s,
            None => continue,
        };

        let node = match s.node.clone() {
            Some(n) => n,
            None => continue,
        };

        let name = match node.name {
            Some(n) => n,
            None => continue,
        };

        let full = name.full;

        let user_pref = name.user_preferred;

        let native = name.native;

        let staff_name = user_pref.unwrap_or(full.unwrap_or(native.unwrap_or(UNKNOWN.to_string())));

        let s_role = s.role.clone();

        let role = s_role.unwrap_or(UNKNOWN.to_string());

        staff_text.push_str(format!("{}: {}", staff_name.as_str(), role.as_str()).as_str());

        i += 1;
    }

    staff_text
}

fn get_character(character: Vec<Option<CharacterEdge>>) -> String {

    let mut character_text = String::new();

    // iterate over staff with index
    let mut i = 0;

    for s in character.into_iter() {

        if i > 4 {

            break;
        }

        let name = match s {
            Some(s) => {

                let node = match s.node {
                    Some(n) => n,
                    None => continue,
                };

                let name = match node.name {
                    Some(n) => n,
                    None => continue,
                };

                let full = name.full;

                let user_pref = name.user_preferred;

                let native = name.native;

                user_pref.unwrap_or(full.unwrap_or(native.unwrap_or(UNKNOWN.to_string())))
            }
            None => UNKNOWN.to_string(),
        };

        character_text.push_str(name.as_str());

        i += 1;
    }

    character_text
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
    db_config: DbConfig,
) -> Result<(), Box<dyn Error>> {

    let is_adult = data.is_adult.unwrap_or(true);

    if is_adult && !get_nsfw(command_interaction, ctx).await {

        return Err(Box::new(error_dispatch::Error::AdultMedia));
    }

    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };

    let media_localised = load_localization_media(guild_id, db_config).await?;

    let mut fields = Vec::new();

    let genres = data.genres.clone().unwrap_or_default();

    // take the first 5 non-optional genres
    let genres = genres
        .into_iter()
        .flatten()
        .take(5)
        .collect::<Vec<String>>();

    let tag = data.tags.clone().unwrap_or_default();

    let tag = tag
        .into_iter()
        .filter_map(|t| {
            if let Some(t) = t {

                Some(t.name)
            } else {

                None
            }
        })
        .take(5)
        .collect::<Vec<String>>();

    fields.push((media_localised.tag, tag.join(", "), true));

    fields.push((media_localised.genre, genres.join(", "), true));

    if let Some(staff) = data.staff.clone() {

        if let Some(edges) = staff.edges {

            let staffs = get_staff(edges);

            fields.push((media_localised.staffs, staffs, true));
        }
    }

    if let Some(characters) = data.characters.clone() {

        if let Some(edges) = characters.edges {

            let characters = get_character(edges);

            fields.push((media_localised.characters, characters, true));
        }
    }

    if let Some(format) = data.format {

        fields.push((media_localised.format, format.to_string(), true))
    }

    if let Some(source) = data.source {

        fields.push((media_localised.source, source.to_string(), true))
    }

    if let Some(start_date) = data.start_date.clone() {

        let mut start_date_str = String::new();

        if let Some(day) = start_date.day {

            start_date_str.push_str(format!("{}/", day).as_str());
        }

        if let Some(month) = start_date.month {

            start_date_str.push_str(format!("{}/", month).as_str());
        }

        if let Some(year) = start_date.year {

            start_date_str.push_str(year.to_string().as_str());
        }

        fields.push((media_localised.start_date, start_date_str, true));
    }

    if let Some(end_date) = data.end_date.clone() {

        let mut end_date_str = String::new();

        if let Some(day) = end_date.day {

            end_date_str.push_str(format!("{}/", day).as_str());
        }

        if let Some(month) = end_date.month {

            end_date_str.push_str(format!("{}/", month).as_str());
        }

        if let Some(year) = end_date.year {

            end_date_str.push_str(year.to_string().as_str());
        }

        fields.push((media_localised.end_date, end_date_str, true));
    }

    if let Some(favourites) = data.favourites {

        fields.push((media_localised.fav, favourites.to_string(), true))
    }

    match data.duration {
        Some(duration) => {

            fields.push((
                media_localised.duration,
                format!("{} {}", duration, media_localised.minutes),
                true,
            ));
        }
        None => {
            if let Some(chapters) = data.chapters {

                fields.push((
                    media_localised.duration,
                    format!("{} {}", chapters, media_localised.chapter),
                    true,
                ));
            }
        }
    }

    let title = match data.title.clone() {
        Some(t) => t,
        None => {
            return Err(Box::new(error_dispatch::Error::Option(String::from(
                "No title",
            ))))
        }
    };

    let mut builder_embed = CreateEmbed::new()
        .timestamp(Timestamp::now())
        .color(COLOR)
        .title(embed_title(&title))
        .url(get_url(&data.clone()))
        .image(get_banner(&data.clone()))
        .fields(fields);

    if let Some(image) = data.cover_image {

        if let Some(extra_large) = image.extra_large {

            builder_embed = builder_embed.thumbnail(extra_large);
        }
    }

    let builder_message = CreateInteractionResponseMessage::new().embed(builder_embed);

    let builder = CreateInteractionResponse::Message(builder_message);

    command_interaction
        .create_response(&ctx.http, builder)
        .await?;

    Ok(())
}
