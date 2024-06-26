use std::fmt::Display;

use serenity::all::CommandInteraction;
use serenity::builder::{CreateInteractionResponse, CreateInteractionResponseMessage};
use serenity::model::Colour;
use serenity::prelude::Context;

use crate::constant::COLOR;
use crate::helper::create_default_embed::get_default_embed;
use crate::helper::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use crate::structure::message::anilist_user::user::{load_localization_user, UserLocalised};

#[cynic::schema("anilist")]
mod schema {}

#[derive(cynic::QueryVariables, Debug, Clone)]
pub struct UserQuerryIdVariables {
    pub id: Option<i32>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Query", variables = "UserQuerryIdVariables")]
pub struct UserQuerryId {
    #[arguments(id: $ id)]
    #[cynic(rename = "User")]
    pub user: Option<User>,
}

#[derive(cynic::QueryVariables, Debug, Clone)]
pub struct UserQuerrySearchVariables<'a> {
    pub search: Option<&'a str>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Query", variables = "UserQuerrySearchVariables")]
pub struct UserQuerrySearch {
    #[arguments(search: $ search)]
    #[cynic(rename = "User")]
    pub user: Option<User>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub avatar: Option<UserAvatar>,
    pub statistics: Option<UserStatisticTypes>,
    pub options: Option<UserOptions>,
    pub banner_image: Option<String>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct UserOptions {
    pub profile_color: Option<String>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct UserStatisticTypes {
    pub anime: Option<UserStatistics>,
    pub manga: Option<UserStatistics2>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "UserStatistics")]
pub struct UserStatistics2 {
    pub count: i32,
    pub mean_score: f64,
    pub standard_deviation: f64,
    pub chapters_read: i32,
    #[arguments(limit: 5, sort: "MEAN_SCORE_DESC")]
    pub tags: Option<Vec<Option<UserTagStatistic>>>,
    #[arguments(limit: 5, sort: "MEAN_SCORE_DESC")]
    pub genres: Option<Vec<Option<UserGenreStatistic>>>,
    #[arguments(sort: "COUNT_DESC")]
    pub statuses: Option<Vec<Option<UserStatusStatistic>>>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct UserStatistics {
    pub count: i32,
    pub mean_score: f64,
    pub standard_deviation: f64,
    pub minutes_watched: i32,
    #[arguments(limit: 5, sort: "MEAN_SCORE_DESC")]
    pub tags: Option<Vec<Option<UserTagStatistic>>>,
    #[arguments(limit: 5, sort: "MEAN_SCORE_DESC")]
    pub genres: Option<Vec<Option<UserGenreStatistic>>>,
    #[arguments(sort: "COUNT_DESC")]
    pub statuses: Option<Vec<Option<UserStatusStatistic>>>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct UserStatusStatistic {
    pub count: i32,
    pub status: Option<MediaListStatus>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct UserGenreStatistic {
    pub genre: Option<String>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct UserTagStatistic {
    pub tag: Option<MediaTag>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct UserAvatar {
    pub large: Option<String>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct MediaTag {
    pub name: String,
}

#[derive(cynic::Enum, Clone, Copy, Debug)]
pub enum MediaListStatus {
    Current,
    Planning,
    Completed,
    Dropped,
    Paused,
    Repeating,
}

#[derive(cynic::Enum, Clone, Copy, Debug)]
pub enum UserStatisticsSort {
    Id,
    IdDesc,
    Count,
    CountDesc,
    Progress,
    ProgressDesc,
    MeanScore,
    MeanScoreDesc,
}

impl Display for MediaListStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MediaListStatus::Current => write!(f, "CURRENT"),
            MediaListStatus::Planning => write!(f, "PLANNING"),
            MediaListStatus::Completed => write!(f, "COMPLETED"),
            MediaListStatus::Dropped => write!(f, "DROPPED"),
            MediaListStatus::Paused => write!(f, "PAUSED"),
            MediaListStatus::Repeating => write!(f, "REPEATING"),
        }
    }
}

impl Display for UserStatisticsSort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserStatisticsSort::Id => write!(f, "ID"),
            UserStatisticsSort::IdDesc => write!(f, "ID_DESC"),
            UserStatisticsSort::Count => write!(f, "COUNT"),
            UserStatisticsSort::CountDesc => write!(f, "COUNT_DESC"),
            UserStatisticsSort::Progress => write!(f, "PROGRESS"),
            UserStatisticsSort::ProgressDesc => write!(f, "PROGRESS_DESC"),
            UserStatisticsSort::MeanScore => write!(f, "MEAN_SCORE"),
            UserStatisticsSort::MeanScoreDesc => write!(f, "MEAN_SCORE_DESC"),
        }
    }
}

/// `send_embed` is an asynchronous function that sends an embed message to a Discord channel.
/// It takes a `Context`, a `CommandInteraction`, and a `UserWrapper` as parameters.
/// `ctx` is the context in which this function is called.
/// `command` is the command interaction that triggered this function.
/// `data` is the user data to be sent in the embed message.
/// It returns a `Result` that contains an empty tuple or an `AppError`.
///
/// This function first gets the guild ID from the command interaction.
/// It then loads the localized user data based on the guild ID.
/// It clones the user data from the `UserWrapper`.
/// It creates a new vector for the fields of the embed message.
/// It clones the manga and anime statistics from the user data.
/// If the user has manga or anime statistics, it pushes the corresponding field to the vector.
/// It then creates an embed message with the user data and the fields.
/// It creates a response message with the embed message.
/// It creates a response with the response message.
/// It sends the response to the Discord channel and returns the result.
///
/// # Arguments
///
/// * `ctx` - The context in which this function is called.
/// * `command` - The command interaction that triggered this function.
/// * `data` - The user data to be sent in the embed message.
///
/// # Returns
///
/// * `Result<(), AppError>` - A Result that contains an empty tuple or an `AppError`.
pub async fn send_embed(
    ctx: &Context,
    command: &CommandInteraction,
    user: User,
    db_type: String,
) -> Result<(), AppError> {
    let guild_id = match command.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };

    let user_localised = load_localization_user(guild_id, db_type).await?;

    let mut field = Vec::new();
    let statistics = user.statistics.clone().unwrap();
    let manga = statistics.manga.clone();
    let anime = statistics.anime.clone();

    if let Some(m) = &manga {
        if m.count > 0 {
            field.push(get_manga_field(user.id, user_localised.clone(), m.clone()))
        }
    }
    if let Some(a) = &anime {
        if a.count > 0 {
            field.push(get_anime_field(user.id, user_localised.clone(), a.clone()))
        }
    }

    let builder_embed = get_default_embed(Some(get_color(user.clone())))
        .title(user.name)
        .url(get_user_url(user.id))
        .fields(field)
        .image(get_banner(&user.id))
        .thumbnail(user.avatar.unwrap().large.unwrap());

    let builder_message = CreateInteractionResponseMessage::new().embed(builder_embed);

    let builder = CreateInteractionResponse::Message(builder_message);

    command
        .create_response(&ctx.http, builder)
        .await
        .map_err(|e| {
            AppError::new(
                format!("Error while sending the command {}", e),
                ErrorType::Command,
                ErrorResponseType::Message,
            )
        })?;
    Ok(())
}

/// `get_user_url` is a function that gets the URL of a user's AniList profile.
/// It takes a `user_id` as a parameter.
/// `user_id` is a 32-bit integer that represents the ID of the user.
/// It returns a `String` that represents the URL of the user's AniList profile.
///
/// # Arguments
///
/// * `user_id` - A 32-bit integer that represents the ID of the user.
///
/// # Returns
///
/// * `String` - A String that represents the URL of the user's AniList profile.
pub fn get_user_url(user_id: i32) -> String {
    format!("https://anilist.co/user/{}", user_id)
}

/// `get_banner` is a function that gets the banner image URL of a user.
/// It takes a `user_id` as a parameter.
/// `user_id` is a reference to a 32-bit integer that represents the ID of the user.
/// It returns a `String` that represents the banner image URL of the user.
///
/// # Arguments
///
/// * `user_id` - A reference to a 32-bit integer that represents the ID of the user.
///
/// # Returns
///
/// * `String` - A String that represents the banner image URL of the user.
pub fn get_banner(user_id: &i32) -> String {
    format!("https://img.anili.st/user/{}", user_id)
}

/// `get_user_manga_url` is a function that gets the URL of a user's manga list on AniList.
/// It takes a `user_id` as a parameter.
/// `user_id` is a 32-bit integer that represents the ID of the user.
/// It returns a `String` that represents the URL of the user's manga list on AniList.
///
/// # Arguments
///
/// * `user_id` - A 32-bit integer that represents the ID of the user.
///
/// # Returns
///
/// * `String` - A String that represents the URL of the user's manga list on AniList.
fn get_user_manga_url(user_id: i32) -> String {
    format!("https://anilist.co/user/{}/mangalist", user_id)
}

/// `get_user_anime_url` is a function that gets the URL of a user's anime list on AniList.
/// It takes a `user_id` as a parameter.
/// `user_id` is a 32-bit integer that represents the ID of the user.
/// It returns a `String` that represents the URL of the user's anime list on AniList.
///
/// # Arguments
///
/// * `user_id` - A 32-bit integer that represents the ID of the user.
///
/// # Returns
///
/// * `String` - A String that represents the URL of the user's anime list on AniList.
fn get_user_anime_url(user_id: i32) -> String {
    format!("https://anilist.co/user/{}/animelist", user_id)
}

/// `get_manga_field` is a function that gets the manga field for a user.
/// It takes a `user_id`, a `UserLocalised`, and a `Manga` as parameters.
/// `user_id` is a 32-bit integer that represents the ID of the user.
/// `localised` is a `UserLocalised` that represents the localized user data.
/// `manga` is a `Manga` that represents the manga statistics of the user.
/// It returns a tuple that contains a `String`, a `String`, and a `bool`.
///
/// # Arguments
///
/// * `user_id` - A 32-bit integer that represents the ID of the user.
/// * `localised` - A `UserLocalised` that represents the localized user data.
/// * `manga` - A `Manga` that represents the manga statistics of the user.
///
/// # Returns
///
/// * `(String, String, bool)` - A tuple that contains a `String`, a `String`, and a `bool`.
fn get_manga_field(
    user_id: i32,
    localised: UserLocalised,
    manga: UserStatistics2,
) -> (String, String, bool) {
    (
        String::new(),
        get_manga_desc(manga, localised, user_id),
        false,
    )
}

/// `get_anime_field` is a function that gets the anime field for a user.
/// It takes a `user_id`, a `UserLocalised`, and an `Anime` as parameters.
/// `user_id` is a 32-bit integer that represents the ID of the user.
/// `localised` is a `UserLocalised` that represents the localized user data.
/// `anime` is an `Anime` that represents the anime statistics of the user.
/// It returns a tuple that contains a `String`, a `String`, and a `bool`.
///
/// # Arguments
///
/// * `user_id` - A 32-bit integer that represents the ID of the user.
/// * `localised` - A `UserLocalised` that represents the localized user data.
/// * `anime` - An `Anime` that represents the anime statistics of the user.
///
/// # Returns
///
/// * `(String, String, bool)` - A tuple that contains a `String`, a `String`, and a `bool`.
fn get_anime_field(
    user_id: i32,
    localised: UserLocalised,
    anime: UserStatistics,
) -> (String, String, bool) {
    (
        String::new(),
        get_anime_desc(anime, localised, user_id),
        false,
    )
}

/// `get_manga_desc` is a function that gets the manga description for a user.
/// It takes a `Manga`, a `UserLocalised`, and a `user_id` as parameters.
/// `manga` is a `Manga` that represents the manga statistics of the user.
/// `localised` is a `UserLocalised` that represents the localized user data.
/// `user_id` is a 32-bit integer that represents the ID of the user.
/// It returns a `String` that represents the manga description of the user.
///
/// # Arguments
///
/// * `manga` - A `Manga` that represents the manga statistics of the user.
/// * `localised` - A `UserLocalised` that represents the localized user data.
/// * `user_id` - A 32-bit integer that represents the ID of the user.
///
/// # Returns
///
/// * `String` - A String that represents the manga description of the user.
fn get_manga_desc(manga: UserStatistics2, localised: UserLocalised, user_id: i32) -> String {
    localised
        .manga
        .replace("$url$", get_user_manga_url(user_id).as_str())
        .replace("$count$", manga.count.to_string().as_str())
        .replace(
            "$complete$",
            get_completed(manga.statuses.unwrap().clone())
                .to_string()
                .as_str(),
        )
        .replace("$chap$", manga.chapters_read.to_string().as_str())
        .replace("$score$", manga.mean_score.to_string().as_str())
        .replace("$sd$", manga.standard_deviation.to_string().as_str())
        .replace(
            "$tag_list$",
            get_tag_list(manga.tags.clone().unwrap()).as_str(),
        )
        .replace(
            "$genre_list$",
            get_genre_list(manga.genres.clone().unwrap()).as_str(),
        )
}

/// `get_tag_list` is a function that gets the tag list for a user.
/// It takes a vector of `Tag` as a parameter.
/// `vec` is a vector of `Tag` that represents the tags of the user.
/// It returns a `String` that represents the tag list of the user.
///
/// # Arguments
///
/// * `vec` - A vector of `Tag` that represents the tags of the user.
///
/// # Returns
///
/// * `String` - A String that represents the tag list of the user.
fn get_tag_list(vec: Vec<Option<UserTagStatistic>>) -> String {
    let vec = vec
        .iter()
        .map(|tag| tag.clone().unwrap().tag.clone().unwrap().name.clone())
        .collect::<Vec<_>>();
    let vec = vec.into_iter().take(5).collect::<Vec<_>>();
    vec.join("/")
}

/// `get_genre_list` is a function that gets the genre list for a user.
/// It takes a vector of `Genre` as a parameter.
/// `vec` is a vector of `Genre` that represents the genres of the user.
/// It returns a `String` that represents the genre list of the user.
///
/// # Arguments
///
/// * `vec` - A vector of `Genre` that represents the genres of the user.
///
/// # Returns
///
/// * `String` - A String that represents the genre list of the user.
fn get_genre_list(vec: Vec<Option<UserGenreStatistic>>) -> String {
    let vec = vec
        .iter()
        .map(|genre| genre.clone().unwrap().genre.as_ref().unwrap().clone())
        .collect::<Vec<_>>();
    let vec = vec.into_iter().take(5).collect::<Vec<_>>();
    vec.join("/")
}

/// `get_completed` is a function that gets the number of completed anime or manga for a user.
/// It takes a vector of `Statuses` as a parameter.
/// `statuses` is a vector of `Statuses` that represents the statuses of the anime or manga of the user.
/// It returns a 32-bit integer that represents the number of completed anime or manga.
///
/// # Arguments
///
/// * `statuses` - A vector of `Statuses` that represents the statuses of the anime or manga of the user.
///
/// # Returns
///
/// * `i32` - A 32-bit integer that represents the number of completed anime or manga.
pub fn get_completed(statuses: Vec<Option<UserStatusStatistic>>) -> i32 {
    let anime_statuses = statuses;
    let mut anime_completed = 0;
    for i in anime_statuses {
        let i = i.unwrap();
        if i.status.unwrap().to_string() == *"COMPLETED" {
            anime_completed = i.count;
        }
    }
    anime_completed
}

/// `get_anime_desc` is a function that gets the anime description for a user.
/// It takes an `Anime`, a `UserLocalised`, and a `user_id` as parameters.
/// `anime` is an `Anime` that represents the anime statistics of the user.
/// `localised` is a `UserLocalised` that represents the localized user data.
/// `user_id` is a 32-bit integer that represents the ID of the user.
/// It returns a `String` that represents the anime description of the user.
///
/// # Arguments
///
/// * `anime` - An `Anime` that represents the anime statistics of the user.
/// * `localised` - A `UserLocalised` that represents the localized user data.
/// * `user_id` - A 32-bit integer that represents the ID of the user.
///
/// # Returns
///
/// * `String` - A String that represents the anime description of the user.
fn get_anime_desc(anime: UserStatistics, localised: UserLocalised, user_id: i32) -> String {
    localised
        .anime
        .replace("$url$", get_user_anime_url(user_id).as_str())
        .replace("$count$", anime.count.to_string().as_str())
        .replace(
            "$complete$",
            get_completed(anime.statuses.clone().unwrap())
                .to_string()
                .as_str(),
        )
        .replace(
            "$duration$",
            get_anime_time_watch(anime.minutes_watched, localised.clone()).as_str(),
        )
        .replace("$score$", anime.mean_score.to_string().as_str())
        .replace("$sd$", anime.standard_deviation.to_string().as_str())
        .replace(
            "$tag_list$",
            get_tag_list(anime.tags.clone().unwrap()).as_str(),
        )
        .replace(
            "$genre_list$",
            get_genre_list(anime.genres.clone().unwrap()).as_str(),
        )
}

/// `get_anime_time_watch` is a function that gets the time watched for a user's anime.
/// It takes a 32-bit integer and a `UserLocalised` as parameters.
/// `i` is a 32-bit integer that represents the minutes watched.
/// `localised1` is a `UserLocalised` that represents the localized user data.
/// It returns a `String` that represents the time watched for the user's anime.
///
/// # Arguments
///
/// * `i` - A 32-bit integer that represents the minutes watched.
/// * `localised1` - A `UserLocalised` that represents the localized user data.
///
/// # Returns
///
/// * `String` - A String that represents the time watched for the user's anime.
fn get_anime_time_watch(i: i32, localised1: UserLocalised) -> String {
    let mut min = i;
    let mut hour = 0;
    let mut days = 0;
    let mut week = 0;

    if min >= 60 {
        hour = min / 60;
        min %= 60;
    }

    if hour >= 24 {
        days = hour / 24;
        hour %= 24;
    }

    if days >= 7 {
        week = days / 7;
        days %= 7;
    }

    let mut tw = String::new();

    let weeks = match week {
        1 => format!("{} {}", localised1.week, week),
        _ => format!("{} {}", localised1.weeks, week),
    };
    tw.push_str(weeks.as_str());

    let days = match days {
        1 => format!("{} {}", localised1.day, days),
        _ => format!("{} {}", localised1.days, days),
    };
    tw.push_str(days.as_str());

    let hours = match hour {
        1 => format!("{} {}", localised1.hour, hour),
        _ => format!("{} {}", localised1.hours, hour),
    };
    tw.push_str(hours.as_str());

    let mins = match min {
        1 => format!("{} {}", localised1.minute, min),
        _ => format!("{} {}", localised1.minutes, min),
    };
    tw.push_str(mins.as_str());

    tw
}

/// `get_color` is a function that gets the color for a user's profile.
/// It takes a `User` as a parameter.
/// `user` is a `User` that represents the user data.
/// It returns a `Colour` that represents the color of the user's profile.
///
/// This function first gets the profile color from the user's options.
/// It then matches the profile color with a predefined set of colors.
/// If the profile color matches a predefined color, it returns the corresponding `Colour`.
/// If the profile color does not match any predefined color, it defaults to a specific hex color code.
///
/// # Arguments
///
/// * `user` - A `User` that represents the user data.
///
/// # Returns
///
/// * `Colour` - A `Colour` that represents the color of the user's profile.
pub fn get_color(user: User) -> Colour {
    let color = match user
        .options
        .unwrap()
        .profile_color
        .clone()
        .unwrap_or_else(|| "#FF00FF".to_string())
        .as_str()
    {
        "blue" => Colour::BLUE,
        "purple" => Colour::PURPLE,
        "pink" => Colour::MEIBE_PINK,
        "orange" => Colour::ORANGE,
        "red" => Colour::RED,
        "green" => Colour::DARK_GREEN,
        "gray" => Colour::LIGHT_GREY,
        _ => COLOR,
    };
    color
}
