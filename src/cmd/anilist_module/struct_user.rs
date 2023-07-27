use serde::Deserialize;
use serde_json::json;
use serenity::utils::Colour;

use crate::cmd::general_module::lang_struct::UserLocalisedText;
use crate::cmd::general_module::request::make_request_anilist;

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
    pub fn get_anime_genre(&self) -> String {
        let mut anime_genre = String::new();
        for i in 0..3 {
            if let Some(genre) = self
                .data
                .user
                .statistics
                .anime
                .genres
                .get(i)
                .and_then(|g| g.genre.as_ref())
            {
                anime_genre.push_str(&format!("{} / ", genre));
            } else {
                anime_genre.push_str("N/A / ");
            }
        }
        anime_genre.pop();
        anime_genre.pop();
        anime_genre
    }

    pub fn get_anime_tag(&self) -> String {
        let mut anime_tag_name = String::new();
        for i in 0..3 {
            if let Some(tags) = self
                .data
                .user
                .statistics
                .anime
                .tags
                .get(i)
                .and_then(|g| g.tag.name.as_ref())
            {
                anime_tag_name.push_str(&format!("{} / ", tags));
            } else {
                anime_tag_name.push_str("N/A / ");
            }
        }
        anime_tag_name.pop();
        anime_tag_name.pop();
        anime_tag_name
    }

    pub fn time_anime_watched(&self, localised_text: UserLocalisedText) -> String {
        let mut min = self
            .data
            .user
            .statistics
            .anime
            .minutes_watched
            .unwrap_or_else(|| 0);
        let mut hour = 0;
        let mut days = 0;
        let mut week = 0;

        if min >= 60 {
            hour = min / 60;
            min = min % 60;
        }

        if hour >= 24 {
            days = hour / 24;
            hour = hour % 24;
        }

        if days >= 7 {
            week = days / 7;
            days = days % 7;
        }

        format!(
            "{}{}{}{}{}{}{}{}",
            week,
            &localised_text.week,
            days,
            &localised_text.day,
            hour,
            &localised_text.hour,
            min,
            &localised_text.minute
        )
    }

    pub fn get_anime_count(&self) -> i32 {
        self.data.user.statistics.anime.count.unwrap_or_else(|| 0)
    }

    pub fn get_anime_score(&self) -> f64 {
        self.data.user.statistics.anime.mean_score.unwrap_or_else(|| 0f64)
    }

    pub fn get_anime_standard_deviation(&self) -> f64 {
        self.data.user.statistics.anime.standard_deviation.unwrap_or_else(|| 0f64)
    }

    pub fn get_anime_completed(&self) -> i32 {
        let anime_statuses = &self.data.user.statistics.anime.statuses;
        let mut anime_completed = 0;
        for i in anime_statuses {
            if i.status == "COMPLETED".to_string() {
                anime_completed = i.count;
            }
        }
        anime_completed
    }


    pub fn get_color(&self) -> Colour {
        let mut _color = Colour::FABLED_PINK;
        match self
            .data
            .user
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

    pub fn get_username(&self) -> String {
        self.data.user.name.clone().unwrap_or_else(|| "N/A".to_string())
    }

    pub fn get_pfp(&self) -> String {
        self.data.user.avatar.large.clone().unwrap_or_else(||
            "https://imgs.search.brave.com/CYnhSvdQcm9aZe3wG84YY0B19zT2wlAuAkiAGu0mcLc/rs:fit:640:400:1/g:ce/aHR0cDovL3d3dy5m/cmVtb250Z3VyZHdh/cmEub3JnL3dwLWNv/bnRlbnQvdXBsb2Fk/cy8yMDIwLzA2L25v/LWltYWdlLWljb24t/Mi5wbmc"
                .to_string())
    }

    pub fn get_banner(&self) -> String {
        format!("https://img.anili.st/user/{}", self.data.user.id.unwrap())
    }


    pub fn get_manga_genre(&self) -> String {
        let mut manga_genre = String::new();
        for i in 0..3 {
            if let Some(genre) = self
                .data
                .user
                .statistics
                .manga
                .genres
                .get(i)
                .and_then(|g| g.genre.as_ref())
            {
                manga_genre.push_str(&format!("{} / ", genre));
            } else {
                manga_genre.push_str("N/A / ");
            }
        }
        manga_genre.pop();
        manga_genre.pop();
        manga_genre
    }

    pub fn get_manga_tag(&self) -> String {
        let mut manga_tag_name = String::new();
        for i in 0..3 {
            if let Some(tags) = self
                .data
                .user
                .statistics
                .manga
                .tags
                .get(i)
                .and_then(|g| g.tag.name.as_ref())
            {
                manga_tag_name.push_str(&format!("{} / ", tags));
            } else {
                manga_tag_name.push_str("N/A / ");
            }
        }
        manga_tag_name.pop();
        manga_tag_name.pop();
        manga_tag_name
    }

    pub fn get_manga_chapter(&self) -> i32 {
        self.data
            .user
            .statistics
            .manga
            .chapters_read
            .unwrap_or_else(|| 0)
    }

    pub fn get_manga_count(&self) -> i32 {
        self.data.user.statistics.manga.count.unwrap_or_else(|| 0)
    }

    pub fn get_manga_score(&self) -> f64 {
        self.data
            .user
            .statistics
            .manga
            .mean_score
            .unwrap_or_else(|| 0f64)
    }

    pub fn get_manga_standard_deviation(&self) -> f64 {
        self.data
            .user
            .statistics
            .manga
            .standard_deviation
            .unwrap_or_else(|| 0f64)
    }

    pub fn get_manga_completed(&self) -> i32 {

        let manga_statuses = &self.data.user.statistics.manga.statuses;
        let mut manga_completed = 0;
        for i in manga_statuses {
            if i.status == "COMPLETED".to_string() {
                manga_completed = i.count;
            }
        }
        manga_completed
    }


    pub fn get_user_url(&self) -> String {
        format!(
            "https://anilist.co/user/{}",
            self.data.user.id.unwrap_or_else(|| 1)
        )
    }

    pub fn get_user_anime_url(&self) -> String {
        format!("{}/animelist", self.get_user_url())
    }

    pub fn get_user_manga_url(&self) -> String {
        format!("{}/mangalist", self.get_user_url())
    }


    pub async fn new_anime_by_id(id: i32) -> Result<UserWrapper, String> {
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
        let data: UserWrapper = match serde_json::from_str(&resp) {
        Ok(result) => result,
        Err(e) => {
            println!("Failed to parse JSON: {}", e);
                return Err(String::from("Error: Failed to retrieve user data"))
            }
        };
        return Ok(data);
    }

    pub async fn new_anime_by_search(search: &String) -> Result<UserWrapper, String> {
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
        let data: UserWrapper = match serde_json::from_str(&resp) {
        Ok(result) => result,
        Err(e) => {
            println!("Failed to parse JSON: {}", e);
                return Err(String::from("Error: Failed to retrieve user data"))
            }
        };
        return Ok(data);
    }
}