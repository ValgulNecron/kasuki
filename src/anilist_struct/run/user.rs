use crate::common::make_anilist_request::make_request_anilist;
use crate::constant::{COLOR, COMMAND_SENDING_ERROR};
use crate::error_enum::AppError;
use crate::error_enum::AppError::MediaGettingError;
use crate::lang_struct::anilist::user::{load_localization_user, UserLocalised};
use serde::Deserialize;
use serde_json::json;
use serenity::all::{
    Colour, CommandInteraction, Context, CreateEmbed, CreateInteractionResponse,
    CreateInteractionResponseMessage, Timestamp,
};

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

impl UserWrapper {
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
        serde_json::from_str(&resp)
            .map_err(|_| MediaGettingError(String::from("Error getting this media.")))
    }

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
        serde_json::from_str(&resp)
            .map_err(|_| MediaGettingError(String::from("Error getting this media.")))
    }
}

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

    match user.statistics.manga.count {
        Some(m) => {
            if m > 0 {
                field.push(get_manga_field(
                    user.id.unwrap_or(0),
                    user_localised.clone(),
                    manga,
                ))
            }
        }
        _ => {}
    }
    match user.statistics.anime.count {
        Some(a) => {
            if a > 0 {
                field.push(get_anime_field(
                    user.id.unwrap_or(0),
                    user_localised.clone(),
                    anime,
                ))
            }
        }
        _ => {}
    }

    let builder_embed = CreateEmbed::new()
        .timestamp(Timestamp::now())
        .color(get_color(user.clone()))
        .title(user.name.unwrap_or(String::new()))
        .url(get_user_url(&user.id.unwrap_or(0)))
        .fields(field)
        .image(get_banner(&user.id.unwrap_or(0)))
        .thumbnail(user.avatar.large.unwrap());

    let builder_message = CreateInteractionResponseMessage::new().embed(builder_embed);

    let builder = CreateInteractionResponse::Message(builder_message);

    command
        .create_response(&ctx.http, builder)
        .await
        .map_err(|_| COMMAND_SENDING_ERROR.clone())
}

pub fn get_user_url(user_id: &i32) -> String {
    format!("https://anilist.co/user/{}", user_id)
}

fn get_banner(user_id: &i32) -> String {
    format!("https://img.anili.st/user/{}", user_id)
}

fn get_user_manga_url(user_id: i32) -> String {
    format!("https://anilist.co/user/{}/mangalist", user_id)
}

fn get_user_anime_url(user_id: i32) -> String {
    format!("https://anilist.co/user/{}/animelist", user_id)
}

fn get_manga_field(user_id: i32, localised: UserLocalised, manga: Manga) -> (String, String, bool) {
    (
        String::new(),
        get_manga_desc(manga, localised, user_id),
        false,
    )
}

fn get_anime_field(user_id: i32, localised: UserLocalised, anime: Anime) -> (String, String, bool) {
    (
        String::new(),
        get_anime_desc(anime, localised, user_id),
        false,
    )
}

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

fn get_tag_list(vec: Vec<Tag>) -> String {
    let vec = vec
        .iter()
        .map(|tag| tag.tag.name.as_ref().unwrap().clone())
        .collect::<Vec<_>>();
    let vec = vec.into_iter().take(5).collect::<Vec<_>>();
    vec.join("/")
}

fn get_genre_list(vec: Vec<Genre>) -> String {
    let vec = vec
        .iter()
        .map(|genre| genre.genre.as_ref().unwrap().clone())
        .collect::<Vec<_>>();
    let vec = vec.into_iter().take(5).collect::<Vec<_>>();
    vec.join("/")
}

fn get_completed(statuses: Vec<Statuses>) -> i32 {
    let anime_statuses = statuses;
    let mut anime_completed = 0;
    for i in anime_statuses {
        if i.status == *"COMPLETED" {
            anime_completed = i.count;
        }
    }
    anime_completed
}

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

    if week == 1 {
        tw.push_str(format!("{} {}", localised1.week, week).as_str())
    } else if week > 1 {
        tw.push_str(format!("{} {}", localised1.weeks, week).as_str())
    }
    if days == 1 {
        tw.push_str(format!("{} {}", localised1.day, days).as_str())
    } else if days > 1 {
        tw.push_str(format!("{} {}", localised1.days, days).as_str())
    }
    if hour == 1 {
        tw.push_str(format!("{} {}", localised1.hour, tw).as_str())
    } else if hour > 1 {
        tw.push_str(format!("{} {}", localised1.hours, tw).as_str())
    }
    if min == 1 {
        tw.push_str(format!("{} {}", localised1.minute, min).as_str())
    } else if min > 1 {
        tw.push_str(format!("{} {}", localised1.minutes, min).as_str())
    }

    tw
}

pub fn get_color(user: User) -> Colour {
    let mut _color = COLOR.clone();
    match user
        .options
        .profile_color
        .clone()
        .unwrap_or_else(|| "#FF00FF".to_string())
        .as_str()
    {
        "blue" => _color = Colour::BLUE,
        "purple" => _color = Colour::PURPLE,
        "pink" => _color = Colour::MEIBE_PINK,
        "orange" => _color = Colour::ORANGE,
        "red" => _color = Colour::RED,
        "green" => _color = Colour::DARK_GREEN,
        "gray" => _color = Colour::LIGHT_GREY,
        _ => {
            _color = {
                let hex_code = "#0D966D";
                let color_code = u32::from_str_radix(&hex_code[1..], 16).unwrap();
                Colour::new(color_code)
            }
        }
    }
    _color
}
