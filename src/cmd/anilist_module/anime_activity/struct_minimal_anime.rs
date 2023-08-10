use serde::Serialize;
use serde_json::json;

use crate::cmd::anilist_module::struct_media::MediaWrapper;
use crate::cmd::general_module::lang_struct::AddActivityLocalisedText;
use crate::cmd::general_module::request::make_request_anilist;

#[derive(Debug, Serialize)]
struct NextAiringEpisode {
    #[serde(rename = "airingAt")]
    airing_at: i64,
    #[serde(rename = "timeUntilAiring")]
    time_until_airing: i64,
    episode: i32,
}

#[derive(Debug, Serialize)]
struct Title {
    romaji: String,
    english: Option<String>,
}

#[derive(Debug, Serialize)]
struct MinimalAnime {
    id: i32,
    title: Title,
    #[serde(rename = "nextAiringEpisode")]
    next_airing_episode: Option<NextAiringEpisode>,
}

#[derive(Debug, Serialize)]
struct MinimalAnimeWrapper {
    data: MinimalAnimeData,
}

#[derive(Debug, Serialize)]
struct MinimalAnimeData {
    media: MinimalAnime,
}

impl MinimalAnimeWrapper {
    pub async fn new_minimal_anime_by_id(localised_text: AddActivityLocalisedText, search: String) -> Result<MediaWrapper, String> {
        let query = "
            query ($name: Int) {
              Media(type: ANIME, id: $name) {
                id
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
        let json = json!({"query": query, "variables": {"search": search}});
        let resp = make_request_anilist(json, false).await;
        // Get json
        let data: MediaWrapper = match serde_json::from_str(&resp) {
            Ok(data) => data,
            Err(error) => {
                println!("Error: {}", error);
                return Err(localised_text.error_no_media.clone());
            }
        };
        return Ok(data);
    }

    pub async fn new_minimal_anime_by_search(localised_text: AddActivityLocalisedText, search: String) -> Result<MediaWrapper, String> {
        let query = "
            query ($name: String) {
              Media(type: ANIME, search: $name) {
                id
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
        let json = json!({"query": query, "variables": {"search": search}});
        let resp = make_request_anilist(json, false).await;
        // Get json
        let data: MediaWrapper = match serde_json::from_str(&resp) {
            Ok(data) => data,
            Err(error) => {
                println!("Error: {}", error);
                return Err(localised_text.error_no_media.clone());
            }
        };
        return Ok(data);
    }
}