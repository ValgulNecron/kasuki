use log::error;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::constant::N_A;
use crate::function::requests::request::make_request_anilist;
use crate::structure::embed::anilist::struct_lang_add_activity::AddActivityLocalisedText;

#[derive(Debug, Deserialize, Serialize, Clone)]
struct NextAiringEpisode {
    #[serde(rename = "airingAt")]
    pub airing_at: Option<i64>,
    #[serde(rename = "timeUntilAiring")]
    time_until_airing: Option<i64>,
    episode: Option<i32>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Title {
    romaji: Option<String>,
    english: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct MinimalAnime {
    id: i32,
    title: Option<Title>,
    #[serde(rename = "nextAiringEpisode")]
    next_airing_episode: Option<NextAiringEpisode>,
    #[serde(rename = "coverImage")]
    cover_image: Option<CoverImage>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MinimalAnimeWrapper {
    data: MinimalAnimeData,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct MinimalAnimeData {
    #[serde(rename = "Media")]
    media: MinimalAnime,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct CoverImage {
    #[serde(rename = "extraLarge")]
    extra_large: Option<String>,
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
                error!("Error: {}", error);
                Err(localised_text.error_no_media.clone())
            }
        }
    }

    pub async fn new_minimal_anime_by_id_no_error(id: i32) -> MinimalAnimeWrapper {
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
        let json = json!({"query": query, "variables": {"name": id}});
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
                error!("Error: {}", error);
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

    pub fn get_en_title(&self) -> &str {
        match &self.data.media.title {
            Some(title) => match &title.english {
                Some(english) => english.as_str(),
                None => N_A,
            },
            None => N_A,
        }
    }

    pub fn get_rj_title(&self) -> &str {
        match &self.data.media.title {
            Some(title) => match &title.romaji {
                Some(romaji) => romaji.as_str(),
                None => N_A,
            },
            None => N_A,
        }
    }

    pub fn get_episode(&self) -> i32 {
        match &self.data.media.next_airing_episode {
            Some(next_airing_episode) => match &next_airing_episode.episode {
                Some(episode) => *episode,
                None => 0,
            },
            None => 0,
        }
    }

    pub fn get_image(&self) -> &str {
        match &self.data.media.cover_image {
            Some(cover_image) => match &cover_image.extra_large {
                Some(extra_large) => extra_large.as_str(),
                None => N_A,
            },
            None => N_A,
        }
    }
}
