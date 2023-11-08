use crate::function::general::html_parser::convert_to_discord_markdown;
use crate::function::general::trim::trim;
use crate::function::requests::request::make_request_anilist;
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Deserialize, Clone)]
pub struct Media {
    pub id: i32,
    pub title: Title,
    #[serde(rename = "meanScore")]
    pub mean_score: i32,
    pub description: String,
    pub tags: Vec<Tag>,
    pub genres: Vec<String>,
    pub format: String,
    pub status: String,
    #[serde(rename = "coverImage")]
    pub cover_image: CoverImage,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Title {
    pub native: String,
    #[serde(rename = "userPreferred")]
    pub user_preferred: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Tag {
    pub name: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Page {
    pub media: Vec<Media>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PageData {
    #[serde(rename = "Page")]
    pub page: Page,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PageWrapper {
    pub data: PageData,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CoverImage {
    #[serde(rename = "extraLarge")]
    pub extra_large: String,
}

impl PageWrapper {
    pub async fn new_anime_page(number: i64) -> PageWrapper {
        let query = "
                    query($anime_page: Int){
                        Page(page: $anime_page, perPage: 1){
                            media(type: ANIME){
                            id
                            title {
                                native
                                userPreferred
                            }
                            meanScore
                            description
                            tags {
                                name
                            }
                            genres
                            format
                            status
                            coverImage {
                                extraLarge
                            }
                        }
                    }
                }";

        let json = json!({"query": query, "variables": {"anime_page": number}});
        let res = make_request_anilist(json, false).await;

        serde_json::from_str(&res).unwrap()
    }

    pub async fn new_manga_page(number: i64) -> PageWrapper {
        let query = "
                    query($manga_page: Int){
                        Page(page: $manga_page, perPage: 1){
                            media(type: MANGA){
                            id
                            title {
                                native
                                userPreferred
                            }
                            meanScore
                            description
                            tags {
                                name
                            }
                            genres
                            format
                            status
                            coverImage {
                                extraLarge
                            }
                        }
                    }
                }";

        let json = json!({"query": query, "variables": {"manga_page": number}});
        let res = make_request_anilist(json, false).await;

        serde_json::from_str(&res).unwrap()
    }

    pub fn get_media(&self) -> Media {
        self.data.page.media[0].clone()
    }

    pub fn get_user_pref_title(&self) -> String {
        self.get_media().title.user_preferred
    }

    pub fn get_native_title(&self) -> String {
        self.get_media().title.native
    }

    pub fn get_genre(&self) -> String {
        self.get_media().genres.join("/")
    }

    pub fn get_tags(&self) -> String {
        self.get_media()
            .tags
            .into_iter()
            .map(|tag| tag.name.clone())
            .collect::<Vec<String>>()
            .join("/")
    }

    pub fn get_cover_image(&self) -> String {
        self.get_media().cover_image.extra_large
    }

    pub fn get_description(&self) -> String {
        let mut desc = self.get_media().description;
        desc = convert_to_discord_markdown(desc);
        let lenght_diff = 4096 - desc.len() as i32;
        if lenght_diff <= 0 {
            trim(desc, lenght_diff)
        } else {
            desc
        }
    }

    pub fn get_format(&self) -> String {
        self.get_media().format
    }

    pub fn get_anime_url(&self) -> String {
        format!("https://anilist.co/anime/{}", self.get_id())
    }

    pub fn get_manga_url(&self) -> String {
        format!("https://anilist.co/manga/{}", self.get_id())
    }

    pub fn get_id(&self) -> i32 {
        self.get_media().id
    }
}
