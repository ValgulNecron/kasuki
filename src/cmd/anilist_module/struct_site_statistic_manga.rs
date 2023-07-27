use serde::Deserialize;

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

}
