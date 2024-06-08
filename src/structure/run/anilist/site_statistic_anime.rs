#[cynic::schema("anilist")]
mod schema {}
#[derive(cynic::QueryVariables, Debug)]
pub struct AnimeStatVariables {
    pub page: Option<i32>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "AnimeStatVariables")]
pub struct AnimeStat {
    #[cynic(rename = "SiteStatistics")]
    pub site_statistics: Option<SiteStatistics>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(variables = "AnimeStatVariables")]
pub struct SiteStatistics {
    #[arguments(page: $page, perPage: 1)]
    pub manga: Option<SiteTrendConnection>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct SiteTrendConnection {
    pub page_info: Option<PageInfo>,
    pub nodes: Option<Vec<Option<SiteTrend>>>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct SiteTrend {
    pub count: i32,
    pub date: i32,
    pub change: i32,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct PageInfo {
    pub total: Option<i32>,
    pub per_page: Option<i32>,
    pub last_page: Option<i32>,
    pub current_page: Option<i32>,
    pub has_next_page: Option<bool>,
}
