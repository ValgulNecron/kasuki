use crate::constant::N_A;
use crate::function::request::request::make_request_anilist;
use crate::structure::embed::anilist::struct_lang_add_activity::AddActivityLocalisedText;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct NextAiringEpisode {
    #[serde(rename = "airingAt")]
    pub airing_at: Option<i64>,
    #[serde(rename = "timeUntilAiring")]
    pub time_until_airing: Option<i64>,
    pub episode: Option<i32>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Title {
    pub romaji: Option<String>,
    pub english: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MinimalAnime {
    pub id: i32,
    pub title: Option<Title>,
    #[serde(rename = "nextAiringEpisode")]
    pub next_airing_episode: Option<NextAiringEpisode>,
    #[serde(rename = "coverImage")]
    pub cover_image: Option<CoverImage>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MinimalAnimeWrapper {
    pub data: MinimalAnimeData,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MinimalAnimeData {
    #[serde(rename = "Media")]
    pub media: MinimalAnime,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CoverImage {
    #[serde(rename = "extraLarge")]
    pub extra_large: Option<String>,
}

impl MinimalAnimeWrapper {
    pub async fn new_minimal_anime_by_id(
        localised_text: AddActivityLocalisedText,
        search: String,
    ) -> Result<MinimalAnimeWrapper, String> {
        let query = "
                query ($name: Int) {
                  Media(type: ANIME, id: $name) {
                    id
                   coverImage {
                      extraLarge
                    }
                    title {
                      romaji
                      english
                    }
                    nextAiringEpisode {
                      airingAt
                      timeUntilAiring
                      episode
                    }
                  }
                }
        ";
        let json = json!({"query": query, "variables": {"name": search}});
        let resp = make_request_anilist(json, true).await;
        // Get json
        match serde_json::from_str(&resp) {
            Ok(data) => data,
            Err(error) => {
                println!("Error: {}", error);
                Err(localised_text.error_no_media.clone())
            }
        }
    }

    pub async fn new_minimal_anime_by_id_no_error(search: String) -> MinimalAnimeWrapper {
        let query = "
            query ($name: Int) {
              Media(type: ANIME, search: $name) {
                id
                coverImage {
                  extraLarge
                }
                title {
                  romaji
                  english
                }
                nextAiringEpisode {
                  airingAt
                  timeUntilAiring
                  episode
                }
              }
        ";
        let json = json!({"query": query, "variables": {"name": search}});
        let resp = make_request_anilist(json, true).await;
        serde_json::from_str(&resp).unwrap()
    }

    pub async fn new_minimal_anime_by_search(
        localised_text: AddActivityLocalisedText,
        search: String,
    ) -> Result<MinimalAnimeWrapper, String> {
        let query = "
            query ($name: String) {
              Media(type: ANIME, search: $name) {
                id
                coverImage {
                  extraLarge
                }
                title {
                  romaji
                  english
                }
                nextAiringEpisode {
                  airingAt
                  timeUntilAiring
                  episode
                }
              }
            }
        ";
        let json = json!({"query": query, "variables": {"name": search}});
        let resp = make_request_anilist(json, true).await;
        // Get json
        match serde_json::from_str(&resp) {
            Ok(data) => data,
            Err(error) => {
                println!("Error: {}", error);
                Err(localised_text.error_no_media.clone())
            }
        }
    }

    pub fn get_id(&self) -> i32 {
        self.data.media.id
    }

    pub fn get_timestamp(&self) -> i64 {
        let media = self.data.media.next_airing_episode.clone().unwrap();
        media.airing_at.unwrap_or(0)
    }

    pub fn get_name(&self) -> String {
        format!("{} / {}", self.get_en_title(), self.get_rj_title())
    }

    pub fn get_en_title(&self) -> String {
        self.data
            .media
            .title
            .clone()
            .unwrap()
            .english
            .clone()
            .unwrap_or(N_A.to_string())
    }

    pub fn get_rj_title(&self) -> String {
        self.data
            .media
            .title
            .clone()
            .unwrap()
            .romaji
            .clone()
            .unwrap_or(N_A.to_string())
    }

    pub fn get_episode(&self) -> i32 {
        self.data
            .media
            .next_airing_episode
            .clone()
            .unwrap()
            .episode
            .unwrap()
    }

    pub fn get_image(&self) -> String {
        self.data
            .media
            .cover_image
            .clone()
            .unwrap()
            .extra_large
            .clone()
            .unwrap()
    }
}
