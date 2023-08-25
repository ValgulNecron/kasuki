use serde::Deserialize;
use serde_json::json;

use crate::cmd::general_module::request::make_request_anilist;

#[derive(Debug, Deserialize)]
pub struct SiteStatisticsAnimeWrapper {
    pub data: SiteStatisticsAnimeData,
}

#[derive(Debug, Deserialize)]
pub struct SiteStatisticsAnimeData {
    #[serde(rename = "SiteStatistics")]
    pub site_statistics: SiteStatisticsAnimeContainer,
}

#[derive(Debug, Deserialize)]
pub struct SiteStatisticsAnimeContainer {
    pub anime: SiteStatisticAnime,
}

#[derive(Debug, Deserialize)]
pub struct SiteStatisticAnime {
    #[serde(rename = "pageInfo")]
    pub page_info: SiteStatisticsAnimePageInfo,
    pub nodes: Vec<SiteStatisticsAnimeNode>,
}

#[derive(Debug, Deserialize)]
pub struct SiteStatisticsAnimePageInfo {
    #[serde(rename = "currentPage")]
    pub current_page: i32,
    #[serde(rename = "lastPage")]
    pub last_page: i32,
    pub total: i32,
    #[serde(rename = "hasNextPage")]
    pub has_next_page: bool,
}

#[derive(Debug, Deserialize)]
pub struct SiteStatisticsAnimeNode {
    pub date: i64,
    pub count: i32,
    pub change: i32,
}

impl SiteStatisticsAnimeWrapper {
    pub async fn new_anime(page_number: i64) -> (SiteStatisticsAnimeWrapper, String) {
        let query = "query($page: Int){
                        SiteStatistics{
                            anime(perPage: 1, page: $page){
                                pageInfo{
                                    currentPage
                                    lastPage
                                    total
                                    hasNextPage
                                }
                                nodes{
                                    date
                                    count
                                    change
                                }
                            }
                        }
                    }
                ";
        let json = json!({"query": query, "variables": {"page": page_number}});
        let res = make_request_anilist(json, false).await;
        let api_response: SiteStatisticsAnimeWrapper = serde_json::from_str(&res).unwrap();
        (api_response, res)
    }
    pub fn has_next_page(&self) -> bool {
        self.data.site_statistics.anime.page_info.has_next_page
    }
}
