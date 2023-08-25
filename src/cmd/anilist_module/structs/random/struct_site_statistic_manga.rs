use serde::Deserialize;
use serde_json::json;

use crate::cmd::general_module::function::request::make_request_anilist;

#[derive(Debug, Deserialize)]
pub struct SiteStatisticsMangaWrapper {
    pub data: SiteStatisticsMangaData,
}

#[derive(Debug, Deserialize)]
pub struct SiteStatisticsMangaData {
    #[serde(rename = "SiteStatistics")]
    pub site_statistics: SiteStatisticsMangaContainer,
}

#[derive(Debug, Deserialize)]
pub struct SiteStatisticsMangaContainer {
    pub manga: SiteStatisticManga,
}

#[derive(Debug, Deserialize)]
pub struct SiteStatisticManga {
    #[serde(rename = "pageInfo")]
    pub page_info: SiteStatisticsMangaPageInfo,
    pub nodes: Vec<SiteStatisticsMangaNode>,
}

#[derive(Debug, Deserialize)]
pub struct SiteStatisticsMangaPageInfo {
    #[serde(rename = "currentPage")]
    pub current_page: i32,
    #[serde(rename = "lastPage")]
    pub last_page: i32,
    pub total: i32,
    #[serde(rename = "hasNextPage")]
    pub has_next_page: bool,
}

#[derive(Debug, Deserialize)]
pub struct SiteStatisticsMangaNode {
    pub date: i64,
    pub count: i32,
    pub change: i32,
}

impl SiteStatisticsMangaWrapper {
    pub async fn new_manga(page_number: i64) -> (SiteStatisticsMangaWrapper, String) {
        let query = "
                    query($page: Int){
                        SiteStatistics{
                            manga(perPage: 1, page: $page){
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
        let api_response: SiteStatisticsMangaWrapper = serde_json::from_str(&res).unwrap();
        (api_response, res)
    }
    pub fn has_next_page(&self) -> bool {
        self.data.site_statistics.manga.page_info.has_next_page
    }
}
