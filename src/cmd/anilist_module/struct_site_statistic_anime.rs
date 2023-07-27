use serde::Deserialize;

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

}