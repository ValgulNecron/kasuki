use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct UserData {
    pub data: UserWrapper,
}

#[derive(Debug, Deserialize, Clone)]
pub struct UserWrapper {
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
    #[serde(rename = "mean_score")]
    pub mean_score: Option<f64>,
    #[serde(rename = "standard_deviation")]
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

pub fn resp_to_user_data(resp: String) -> Result<UserData, String> {
    match serde_json::from_str(&resp) {
        Ok(result) => Ok(result),
        Err(e) => {
            println!("Failed to parse JSON: {}", e);
            Err(String::from("Error: Failed to retrieve user data"))
        }
    }
}
