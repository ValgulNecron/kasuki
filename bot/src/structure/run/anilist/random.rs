use std::fmt::Display;

#[cynic::schema("anilist")]

mod schema {}

#[derive(cynic::QueryVariables, Debug, Clone)]

pub struct RandomPageMediaVariables {
    pub media_type: Option<MediaType>,
    pub page: Option<i32>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Query", variables = "RandomPageMediaVariables")]

pub struct RandomPageMedia {
    #[arguments(perPage: 1, page: $ page)]
    #[cynic(rename = "Page")]
    pub page: Option<Page>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(variables = "RandomPageMediaVariables")]

pub struct Page {
    #[arguments(type: $ media_type)]
    pub media: Option<Vec<Option<Media>>>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]

pub struct Media {
    pub id: i32,
    pub title: Option<MediaTitle>,
    pub description: Option<String>,
    pub mean_score: Option<i32>,
    pub tags: Option<Vec<Option<MediaTag>>>,
    pub genres: Option<Vec<Option<String>>>,
    pub format: Option<MediaFormat>,
    pub status: Option<MediaStatus>,
    pub cover_image: Option<MediaCoverImage>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]

pub struct MediaCoverImage {
    pub extra_large: Option<String>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]

pub struct MediaTag {
    pub name: String,
}

#[derive(cynic::QueryFragment, Debug, Clone)]

pub struct MediaTitle {
    pub native: Option<String>,
    pub user_preferred: Option<String>,
}

#[derive(cynic::Enum, Clone, Copy, Debug)]

pub enum MediaFormat {
    Tv,
    TvShort,
    Movie,
    Special,
    Ova,
    Ona,
    Music,
    Manga,
    Novel,
    OneShot,
}

#[derive(cynic::Enum, Clone, Copy, Debug)]

pub enum MediaStatus {
    Finished,
    Releasing,
    NotYetReleased,
    Cancelled,
    Hiatus,
}

#[derive(cynic::Enum, Clone, Copy, Debug)]

pub enum MediaType {
    Anime,
    Manga,
}

impl Display for MediaFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {

        match self {
            MediaFormat::Tv => write!(f, "TV"),
            MediaFormat::TvShort => write!(f, "TV_SHORT"),
            MediaFormat::Movie => write!(f, "MOVIE"),
            MediaFormat::Special => write!(f, "SPECIAL"),
            MediaFormat::Ova => write!(f, "OVA"),
            MediaFormat::Ona => write!(f, "ONA"),
            MediaFormat::Music => write!(f, "MUSIC"),
            MediaFormat::Manga => write!(f, "MANGA"),
            MediaFormat::Novel => write!(f, "NOVEL"),
            MediaFormat::OneShot => write!(f, "ONE_SHOT"),
        }
    }
}

impl Display for MediaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {

        match self {
            MediaStatus::Finished => write!(f, "FINISHED"),
            MediaStatus::Releasing => write!(f, "RELEASING"),
            MediaStatus::NotYetReleased => write!(f, "NOT_YET_RELEASED"),
            MediaStatus::Cancelled => write!(f, "CANCELLED"),
            MediaStatus::Hiatus => write!(f, "HIATUS"),
        }
    }
}
