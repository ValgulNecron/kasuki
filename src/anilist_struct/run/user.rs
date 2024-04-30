use serde::Deserialize;
use serde_json::json;
use serenity::all::{
    Colour, CommandInteraction, Context, CreateInteractionResponse,
    CreateInteractionResponseMessage,
};

use crate::common::default_embed::get_default_embed;
use crate::common::make_anilist_request::make_request_anilist;
use crate::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use crate::lang_struct::anilist_user::user::{load_localization_user, UserLocalised};

#[derive(Debug, Deserialize, Clone)]
pub struct UserWrapper {
    pub data: UserData,
}

#[derive(Debug, Deserialize, Clone)]
pub struct UserData {
    #[serde(rename = "User")]
    pub user: User,
}

#[derive(Debug, Deserialize, Clone)]
pub struct User {
    pub id: Option<i32>,
    pub name: Option<String>,
    pub avatar: Avatar,
    pub statistics: Statistics,
    pub options: Options,
    #[serde(rename = "bannerImage")]
    pub banner_image: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Options {
    #[serde(rename = "profileColor")]
    pub profile_color: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Avatar {
    pub large: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Statistics {
    pub anime: Anime,
    pub manga: Manga,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Anime {
    pub count: Option<i32>,
    #[serde(rename = "meanScore")]
    pub mean_score: Option<f64>,
    #[serde(rename = "standardDeviation")]
    pub standard_deviation: Option<f64>,
    #[serde(rename = "minutesWatched")]
    pub minutes_watched: Option<i32>,
    pub tags: Vec<Tag>,
    pub genres: Vec<Genre>,
    pub statuses: Vec<Statuses>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Manga {
    pub count: Option<i32>,
    #[serde(rename = "meanScore")]
    pub mean_score: Option<f64>,
    #[serde(rename = "standardDeviation")]
    pub standard_deviation: Option<f64>,
    #[serde(rename = "chaptersRead")]
    pub chapters_read: Option<i32>,
    pub tags: Vec<Tag>,
    pub genres: Vec<Genre>,
    pub statuses: Vec<Statuses>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Statuses {
    pub count: i32,
    pub status: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Tag {
    pub tag: TagData,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TagData {
    pub name: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Genre {
    pub genre: Option<String>,
}

/// `UserWrapper` is an implementation block for the `UserWrapper` struct.
impl UserWrapper {
    /// `new_user_by_id` is an asynchronous function that creates a new user by ID.
    /// It takes an `id` as a parameter.
    /// `id` is a 32-bit integer that represents the ID of the user.
    /// It returns a `Result` that contains a `UserWrapper` or an `AppError`.
    ///
    /// This function first defines a GraphQL query string that takes an `id` as a variable.
    /// It then creates a JSON object with the query string and the variable.
    /// The `id` variable is set to the `id` parameter.
    /// It makes a request to AniList with the JSON object and waits for the response.
    /// It then deserializes the response into a `UserWrapper` and returns it.
    ///
    /// # Arguments
    ///
    /// * `id` - A 32-bit integer that represents the ID of the user.
    ///
    /// # Returns
    ///
    /// * `Result<UserWrapper, AppError>` - A Result that contains a `UserWrapper` or an `AppError`.
    pub async fn new_user_by_id(id: i32) -> Result<UserWrapper, AppError> {
        let query_id: &str = "
query ($name: Int, $limit: Int = 5) {
  User(id: $name) {
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
        let json = json!({"query": query_id, "variables": {"name": id}});
        let resp = make_request_anilist(json, true).await;
        serde_json::from_str(&resp).map_err(|e| {
            AppError::new(
                format!("Error getting the user with id {}. {}", id, e),
                ErrorType::WebRequest,
                ErrorResponseType::Message,
            )
        })
    }

    /// `new_user_by_search` is an asynchronous function that creates a new user by search.
    /// It takes a `search` as a parameter.
    /// `search` is a reference to a String that represents the search query.
    /// It returns a `Result` that contains a `UserWrapper` or an `AppError`.
    ///
    /// This function first defines a GraphQL query string that takes a `search` as a variable.
    /// It then creates a JSON object with the query string and the variable.
    /// The `search` variable is set to the `search` parameter.
    /// It makes a request to AniList with the JSON object and waits for the response.
    /// It then deserializes the response into a `UserWrapper` and returns it.
    ///
    /// # Arguments
    ///
    /// * `search` - A reference to a String that represents the search query.
    ///
    /// # Returns
    ///
    /// * `Result<UserWrapper, AppError>` - A Result that contains a `UserWrapper` or an `AppError`.
    pub async fn new_user_by_search(search: &String) -> Result<UserWrapper, AppError> {
        let query_string: &str = "
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
        let json = json!({"query": query_string, "variables": {"name": search}});
        let resp = make_request_anilist(json, true).await;
        serde_json::from_str(&resp).map_err(|e| {
            AppError::new(
                format!("Error getting the user with name {}. {}", search, e),
                ErrorType::WebRequest,
                ErrorResponseType::Message,
            )
        })
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
    data: UserWrapper,
) -> Result<(), AppError> {
    let guild_id = match command.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };

    let user_localised = load_localization_user(guild_id).await?;

    let user = data.data.user.clone();

    let mut field = Vec::new();

    let manga = user.statistics.manga.clone();
    let anime = user.statistics.anime.clone();

    if let Some(m) = user.statistics.manga.count {
        if m > 0 {
            field.push(get_manga_field(
                user.id.unwrap_or(0),
                user_localised.clone(),
                manga,
            ))
        }
    }
    if let Some(a) = user.statistics.anime.count {
        if a > 0 {
            field.push(get_anime_field(
                user.id.unwrap_or(0),
                user_localised.clone(),
                anime,
            ))
        }
    }

    let builder_embed = get_default_embed(Some(get_color(user.clone())))
        .title(user.name.unwrap_or_default())
        .url(get_user_url(user.id.unwrap_or(0)))
        .fields(field)
        .image(get_banner(&user.id.unwrap_or(0)))
        .thumbnail(user.avatar.large.unwrap());

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
fn get_manga_field(user_id: i32, localised: UserLocalised, manga: Manga) -> (String, String, bool) {
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
fn get_anime_field(user_id: i32, localised: UserLocalised, anime: Anime) -> (String, String, bool) {
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
fn get_manga_desc(manga: Manga, localised: UserLocalised, user_id: i32) -> String {
    localised
        .manga
        .replace("$url$", get_user_manga_url(user_id).as_str())
        .replace("$count$", manga.count.unwrap_or(0).to_string().as_str())
        .replace(
            "$complete$",
            get_completed(manga.statuses.clone()).to_string().as_str(),
        )
        .replace(
            "$chap$",
            manga.chapters_read.unwrap_or(0).to_string().as_str(),
        )
        .replace(
            "$score$",
            manga.mean_score.unwrap_or(0f64).to_string().as_str(),
        )
        .replace(
            "$sd$",
            manga
                .standard_deviation
                .unwrap_or(0f64)
                .to_string()
                .as_str(),
        )
        .replace("$tag_list$", get_tag_list(manga.tags.clone()).as_str())
        .replace(
            "$genre_list$",
            get_genre_list(manga.genres.clone()).as_str(),
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
fn get_tag_list(vec: Vec<Tag>) -> String {
    let vec = vec
        .iter()
        .map(|tag| tag.tag.name.as_ref().unwrap().clone())
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
fn get_genre_list(vec: Vec<Genre>) -> String {
    let vec = vec
        .iter()
        .map(|genre| genre.genre.as_ref().unwrap().clone())
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
pub fn get_completed(statuses: Vec<Statuses>) -> i32 {
    let anime_statuses = statuses;
    let mut anime_completed = 0;
    for i in anime_statuses {
        if i.status == *"COMPLETED" {
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
fn get_anime_desc(anime: Anime, localised: UserLocalised, user_id: i32) -> String {
    localised
        .anime
        .replace("$url$", get_user_anime_url(user_id).as_str())
        .replace("$count$", anime.count.unwrap_or(0).to_string().as_str())
        .replace(
            "$complete$",
            get_completed(anime.statuses.clone()).to_string().as_str(),
        )
        .replace(
            "$duration$",
            get_anime_time_watch(anime.minutes_watched.unwrap_or(0), localised.clone()).as_str(),
        )
        .replace(
            "$score$",
            anime.mean_score.unwrap_or(0f64).to_string().as_str(),
        )
        .replace(
            "$sd$",
            anime
                .standard_deviation
                .unwrap_or(0f64)
                .to_string()
                .as_str(),
        )
        .replace("$tag_list$", get_tag_list(anime.tags.clone()).as_str())
        .replace(
            "$genre_list$",
            get_genre_list(anime.genres.clone()).as_str(),
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
        _ => {
            let hex_code = "#0D966D";
            let color_code = u32::from_str_radix(&hex_code[1..], 16).unwrap();
            Colour::new(color_code)
        }
    };
    color
}
