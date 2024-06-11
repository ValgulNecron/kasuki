#[cynic::schema("anilist")]
mod schema {}
#[derive(cynic::QueryVariables, Debug, Clone)]
pub struct StudioQuerryVariables<'a> {
    pub id: Option<i32>,
    pub search: Option<&'a str>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Query", variables = "StudioQuerryVariables")]
pub struct StudioQuerry {
    #[arguments(id: $id, search: $search)]
    #[cynic(rename = "Studio")]
    pub studio: Option<Studio>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct Studio {
    pub id: i32,
    pub name: String,
    pub is_animation_studio: bool,
    pub site_url: Option<String>,
    pub favourites: Option<i32>,
    #[arguments(perPage: 15, sort: "START_DATE_DESC")]
    pub media: Option<MediaConnection>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct MediaConnection {
    pub nodes: Option<Vec<Option<Media>>>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct Media {
    pub title: Option<MediaTitle>,
    pub site_url: Option<String>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct MediaTitle {
    pub romaji: Option<String>,
    pub user_preferred: Option<String>,
}

#[derive(cynic::Enum, Clone, Copy, Debug)]
pub enum MediaSort {
    Id,
    IdDesc,
    TitleRomaji,
    TitleRomajiDesc,
    TitleEnglish,
    TitleEnglishDesc,
    TitleNative,
    TitleNativeDesc,
    Type,
    TypeDesc,
    Format,
    FormatDesc,
    StartDate,
    StartDateDesc,
    EndDate,
    EndDateDesc,
    Score,
    ScoreDesc,
    Popularity,
    PopularityDesc,
    Trending,
    TrendingDesc,
    Episodes,
    EpisodesDesc,
    Duration,
    DurationDesc,
    Status,
    StatusDesc,
    Chapters,
    ChaptersDesc,
    Volumes,
    VolumesDesc,
    UpdatedAt,
    UpdatedAtDesc,
    SearchMatch,
    Favourites,
    FavouritesDesc,
}
