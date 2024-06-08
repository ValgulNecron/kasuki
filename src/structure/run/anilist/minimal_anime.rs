#[cynic::schema("anilist")]
mod schema {}
#[derive(cynic::QueryVariables, Debug, Clone)]
pub struct MinimalAnimeDataIdVariables {
    pub id: Option<i32>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Query", variables = "MinimalAnimeDataIdVariables")]
pub struct MinimalAnimeDataId {
    #[arguments(id: $id)]
    #[cynic(rename = "Media")]
    pub media: Option<Media>,
}

#[derive(cynic::QueryVariables, Debug, Clone)]
pub struct MinimalAnimeDataSearchVariables<'a> {
    pub search: Option<&'a str>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Query", variables = "MinimalAnimeDataSearchVariables")]
pub struct MinimalAnimeDataSearch {
    #[arguments(search: $search)]
    #[cynic(rename = "Media")]
    pub media: Option<Media>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct Media {
    pub id: i32,
    pub cover_image: Option<MediaCoverImage>,
    pub title: Option<MediaTitle>,
    pub next_airing_episode: Option<AiringSchedule>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct MediaTitle {
    pub english: Option<String>,
    pub native: Option<String>,
    pub romaji: Option<String>,
    pub user_preferred: Option<String>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct MediaCoverImage {
    pub extra_large: Option<String>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct AiringSchedule {
    pub airing_at: i32,
    pub time_until_airing: i32,
    pub episode: i32,
}
