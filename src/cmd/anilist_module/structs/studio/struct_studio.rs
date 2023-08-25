use serde::Deserialize;
use serde_json::json;

use crate::cmd::general_module::lang_struct::StudioLocalisedText;
use crate::cmd::general_module::function::request::make_request_anilist;

#[derive(Debug, Deserialize, Clone)]
pub struct Title {
    romaji: String,
    #[serde(rename = "userPreferred")]
    user_preferred: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MediaNode {
    title: Title,
    #[serde(rename = "siteUrl")]
    site_url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Media {
    nodes: Vec<MediaNode>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Studio {
    pub id: u32,
    pub name: String,
    #[serde(rename = "isAnimationStudio")]
    pub is_animation_studio: bool,
    #[serde(rename = "siteUrl")]
    pub site_url: String,
    pub favourites: u32,
    pub media: Media,
}

#[derive(Debug, Deserialize, Clone)]
pub struct StudioData {
    #[serde(rename = "Studio")]
    pub studio: Studio,
}

#[derive(Debug, Deserialize, Clone)]
pub struct StudioWrapper {
    pub data: StudioData,
}

impl StudioWrapper {
    pub async fn new_studio_by_id(id: i32) -> Result<StudioWrapper, String> {
        let query_id: &str = "\
        query ($name: Int, $limit: Int = 15) {
          Studio(id: $name) {
            id
            name
            isAnimationStudio
            siteUrl
            favourites
            media(perPage: $limit, sort: START_DATE_DESC) {
              nodes {
                title{
                  romaji
                  userPreferred
                }
                siteUrl
              }
            }
          }
        }
        ";
        let json = json!({"query": query_id, "variables": {"name": id}});
        let resp = make_request_anilist(json, false).await;
        let data: StudioWrapper = match serde_json::from_str(&resp) {
            Ok(result) => result,
            Err(e) => {
                println!("Failed to parse JSON: {}", e);
                return Err(String::from("Error: Failed to retrieve user data"));
            }
        };
        return Ok(data);
    }

    pub async fn new_studio_by_search(search: &String) -> Result<StudioWrapper, String> {
        let query_string: &str = "
        query ($name: String, $limit: Int = 5) {
          Studio(search: $name) {
            id
            name
            isAnimationStudio
            siteUrl
            favourites
            media(perPage: $limit, sort: START_DATE_DESC) {
              nodes {
                title{
                  romaji
                  userPreferred
                }
                siteUrl
              }
            }
          }
        }
        ";
        let json = json!({"query": query_string, "variables": {"name": search}});
        let resp = make_request_anilist(json, false).await;
        let data: StudioWrapper = match serde_json::from_str(&resp) {
            Ok(result) => result,
            Err(e) => {
                println!("Failed to parse JSON: {}", e);
                return Err(String::from("Error: Failed to retrieve user data"));
            }
        };
        return Ok(data);
    }

    pub fn get_studio_name(&self) -> String {
        return self.data.studio.name.clone();
    }

    pub fn get_site_url(&self) -> String {
        return self.data.studio.site_url.clone();
    }

    pub fn get_favourite(&self) -> String {
        return self.data.studio.favourites.to_string();
    }

    pub fn get_anime_manga_list(&self, localised_text: StudioLocalisedText) -> String {
        let list = self.data.studio.media.nodes.clone();
        let mut content = format!("{} \n", localised_text.anime_or_manga);
        for m in list {
            content.push_str(self.get_one_anime_manga(m).as_str())
        }

        return content;
    }
    pub fn get_one_anime_manga(&self, m: MediaNode) -> String {
        let anime_manga = format!(
            "[{} / {}]({}) \n \n",
            m.title.romaji, m.title.user_preferred, m.site_url
        );
        return anime_manga;
    }

    pub fn get_desc(&self, localised_text: StudioLocalisedText) -> String {
        let desc = format!(
            "id: {} \n {}{} \n {} \n \n \n ",
            self.get_id(),
            localised_text.favorite,
            self.get_favourite(),
            self.get_anime_manga_list(localised_text.clone())
        );

        return desc;
    }

    pub fn get_id(&self) -> u32 {
        self.data.studio.id
    }
}