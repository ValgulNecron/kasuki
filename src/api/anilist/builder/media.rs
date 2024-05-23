use std::fmt::Display;

use crate::api::anilist::response::date::FuzzyDate;

#[derive(Debug, Clone)]
pub struct MediaAPIBuilder {
    pub id: Option<i32>,
    pub id_mal: Option<i32>,
    pub start_date: Option<FuzzyDate>,
    pub end_date: Option<FuzzyDate>,
    pub season: Option<MediaAPISeason>,
    pub season_year: Option<i32>,
    pub media_type: Option<MediaAPIType>,
    pub format: Option<MediaAPIFormat>,
    pub status: Option<MediaAPIStatus>,
    pub episodes: Option<i32>,
    pub duration: Option<i32>,
    pub chapters: Option<i32>,
    pub volumes: Option<i32>,
    pub is_adult: Option<bool>,
    pub genre: Option<String>,
    pub tag: Option<String>,
    pub minimum_tag_rank: Option<i32>,
    pub tag_category: Option<String>,
    pub licensed_by: Option<String>,
    pub licensed_by_id: Option<i32>,
    pub average_score: Option<i32>,
    pub popularity: Option<i32>,
    pub source: Option<MediaAPISource>,
    pub country_of_origin: Option<String>,
    pub search: Option<String>,
    pub id_not: Option<i32>,
    pub id_in: Option<Vec<i32>>,
    pub id_not_in: Option<Vec<i32>>,
    pub id_mal_not: Option<i32>,
    pub id_mal_in: Option<Vec<i32>>,
    pub id_mal_not_in: Option<Vec<i32>>,
    pub start_date_greater: Option<FuzzyDate>,
    pub start_date_lesser: Option<FuzzyDate>,
    pub start_date_like: Option<String>,
    pub end_date_greater: Option<FuzzyDate>,
    pub end_date_lesser: Option<FuzzyDate>,
    pub end_date_like: Option<String>,
    pub format_in: Option<Vec<MediaAPIFormat>>,
    pub format_not: Option<MediaAPIFormat>,
    pub format_not_in: Option<Vec<MediaAPIFormat>>,
    pub status_in: Option<Vec<MediaAPIStatus>>,
    pub status_not: Option<MediaAPIStatus>,
    pub status_not_in: Option<Vec<MediaAPIStatus>>,
    pub episodes_greater: Option<i32>,
    pub episodes_lesser: Option<i32>,
    pub duration_greater: Option<i32>,
    pub duration_lesser: Option<i32>,
    pub chapters_greater: Option<i32>,
    pub chapters_lesser: Option<i32>,
    pub volumes_greater: Option<i32>,
    pub volumes_lesser: Option<i32>,
    pub genre_in: Option<Vec<String>>,
    pub genre_not_in: Option<Vec<String>>,
    pub tag_in: Option<Vec<String>>,
    pub tag_not_in: Option<Vec<String>>,
    pub tag_category_in: Option<Vec<String>>,
    pub tag_category_not_in: Option<Vec<String>>,
    pub licensed_by_in: Option<Vec<String>>,
    pub licensed_by_id_in: Option<Vec<i32>>,
    pub average_score_not: Option<i32>,
    pub average_score_greater: Option<i32>,
    pub average_score_lesser: Option<i32>,
    pub popularity_not: Option<i32>,
    pub popularity_greater: Option<i32>,
    pub popularity_lesser: Option<i32>,
    pub source_in: Option<Vec<MediaAPISource>>,
    pub sort: Option<MediaAPISort>,
    pub query: Option<String>,
}

#[derive(Debug, Clone)]
pub enum MediaAPISource {
    ORIGINAL,
    MANGA,
    LightNovel,
    VisualNovel,
    VideoGame,
    OTHER,
    NOVEL,
    DOUJINSHI,
    ANIME,
    WebNovel,
    LiveAction,
    GAME,
    COMIC,
    MultimediaProject,
    PictureBook,
}

impl From<String> for MediaAPISource {
    fn from(value: String) -> Self {
        match value.as_str() {
            "ORIGINAL" => MediaAPISource::ORIGINAL,
            "MANGA" => MediaAPISource::MANGA,
            "LIGHT_NOVEL" => MediaAPISource::LightNovel,
            "VISUAL_NOVEL" => MediaAPISource::VisualNovel,
            "VIDEO_GAME" => MediaAPISource::VideoGame,
            "OTHER" => MediaAPISource::OTHER,
            "NOVEL" => MediaAPISource::NOVEL,
            "DOUJINSHI" => MediaAPISource::DOUJINSHI,
            "ANIME" => MediaAPISource::ANIME,
            "WEB_NOVEL" => MediaAPISource::WebNovel,
            "LIVE_ACTION" => MediaAPISource::LiveAction,
            "GAME" => MediaAPISource::GAME,
            "COMIC" => MediaAPISource::COMIC,
            "MULTIMEDIA_PROJECT" => MediaAPISource::MultimediaProject,
            "PICTURE_BOOK" => MediaAPISource::PictureBook,
            _ => MediaAPISource::ORIGINAL,
        }
    }
}

impl From<MediaAPISource> for String {
    fn from(value: MediaAPISource) -> Self {
        match value {
            MediaAPISource::ORIGINAL => "ORIGINAL".to_string(),
            MediaAPISource::MANGA => "MANGA".to_string(),
            MediaAPISource::LightNovel => "LIGHT_NOVEL".to_string(),
            MediaAPISource::VisualNovel => "VISUAL_NOVEL".to_string(),
            MediaAPISource::VideoGame => "VIDEO_GAME".to_string(),
            MediaAPISource::OTHER => "OTHER".to_string(),
            MediaAPISource::NOVEL => "NOVEL".to_string(),
            MediaAPISource::DOUJINSHI => "DOUJINSHI".to_string(),
            MediaAPISource::ANIME => "ANIME".to_string(),
            MediaAPISource::WebNovel => "WEB_NOVEL".to_string(),
            MediaAPISource::LiveAction => "LIVE_ACTION".to_string(),
            MediaAPISource::GAME => "GAME".to_string(),
            MediaAPISource::COMIC => "COMIC".to_string(),
            MediaAPISource::MultimediaProject => "MULTIMEDIA_PROJECT".to_string(),
            MediaAPISource::PictureBook => "PICTURE_BOOK".to_string(),
        }
    }
}

impl Display for MediaAPISource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MediaAPISource::ORIGINAL => write!(f, "ORIGINAL"),
            MediaAPISource::MANGA => write!(f, "MANGA"),
            MediaAPISource::LightNovel => write!(f, "LIGHT_NOVEL"),
            MediaAPISource::VisualNovel => write!(f, "VISUAL_NOVEL"),
            MediaAPISource::VideoGame => write!(f, "VIDEO_GAME"),
            MediaAPISource::OTHER => write!(f, "OTHER"),
            MediaAPISource::NOVEL => write!(f, "NOVEL"),
            MediaAPISource::DOUJINSHI => write!(f, "DOUJINSHI"),
            MediaAPISource::ANIME => write!(f, "ANIME"),
            MediaAPISource::WebNovel => write!(f, "WEB_NOVEL"),
            MediaAPISource::LiveAction => write!(f, "LIVE_ACTION"),
            MediaAPISource::GAME => write!(f, "GAME"),
            MediaAPISource::COMIC => write!(f, "COMIC"),
            MediaAPISource::MultimediaProject => write!(f, "MULTIMEDIA_PROJECT"),
            MediaAPISource::PictureBook => write!(f, "PICTURE_BOOK"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum MediaAPIStatus {
    FINISHED,
    RELEASING,
    NotYetReleased,
    CANCELLED,
    HIATUS,
}

impl From<String> for MediaAPIStatus {
    fn from(value: String) -> Self {
        match value.as_str() {
            "FINISHED" => MediaAPIStatus::FINISHED,
            "RELEASING" => MediaAPIStatus::RELEASING,
            "NOT_YET_RELEASED" => MediaAPIStatus::NotYetReleased,
            "CANCELLED" => MediaAPIStatus::CANCELLED,
            "HIATUS" => MediaAPIStatus::HIATUS,
            _ => MediaAPIStatus::FINISHED,
        }
    }
}

impl From<MediaAPIStatus> for String {
    fn from(status: MediaAPIStatus) -> Self {
        match status {
            MediaAPIStatus::FINISHED => "FINISHED".to_string(),
            MediaAPIStatus::RELEASING => "RELEASING".to_string(),
            MediaAPIStatus::NotYetReleased => "NOT_YET_RELEASED".to_string(),
            MediaAPIStatus::CANCELLED => "CANCELLED".to_string(),
            MediaAPIStatus::HIATUS => "HIATUS".to_string(),
        }
    }
}

impl Display for MediaAPIStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MediaAPIStatus::FINISHED => write!(f, "FINISHED"),
            MediaAPIStatus::RELEASING => write!(f, "RELEASING"),
            MediaAPIStatus::NotYetReleased => write!(f, "NOT_YET_RELEASED"),
            MediaAPIStatus::CANCELLED => write!(f, "CANCELLED"),
            MediaAPIStatus::HIATUS => write!(f, "HIATUS"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum MediaAPIFormat {
    TV,
    TvShort,
    MOVIE,
    SPECIAL,
    OVA,
    ONA,
    MUSIC,
    MANGA,
    NOVEL,
    OneShot,
}

impl From<String> for MediaAPIFormat {
    fn from(value: String) -> Self {
        match value.as_str() {
            "TV" => MediaAPIFormat::TV,
            "TV_SHORT" => MediaAPIFormat::TvShort,
            "MOVIE" => MediaAPIFormat::MOVIE,
            "SPECIAL" => MediaAPIFormat::SPECIAL,
            "OVA" => MediaAPIFormat::OVA,
            "ONA" => MediaAPIFormat::ONA,
            "MUSIC" => MediaAPIFormat::MUSIC,
            "MANGA" => MediaAPIFormat::MANGA,
            "NOVEL" => MediaAPIFormat::NOVEL,
            "ONE_SHOT" => MediaAPIFormat::OneShot,
            _ => MediaAPIFormat::TV,
        }
    }
}

impl From<MediaAPIFormat> for String {
    fn from(format: MediaAPIFormat) -> Self {
        match format {
            MediaAPIFormat::TV => "TV".to_string(),
            MediaAPIFormat::TvShort => "TV_SHORT".to_string(),
            MediaAPIFormat::MOVIE => "MOVIE".to_string(),
            MediaAPIFormat::SPECIAL => "SPECIAL".to_string(),
            MediaAPIFormat::OVA => "OVA".to_string(),
            MediaAPIFormat::ONA => "ONA".to_string(),
            MediaAPIFormat::MUSIC => "MUSIC".to_string(),
            MediaAPIFormat::MANGA => "MANGA".to_string(),
            MediaAPIFormat::NOVEL => "NOVEL".to_string(),
            MediaAPIFormat::OneShot => "ONE_SHOT".to_string(),
        }
    }
}

impl Display for MediaAPIFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MediaAPIFormat::TV => write!(f, "TV"),
            MediaAPIFormat::TvShort => write!(f, "TV_SHORT"),
            MediaAPIFormat::MOVIE => write!(f, "MOVIE"),
            MediaAPIFormat::SPECIAL => write!(f, "SPECIAL"),
            MediaAPIFormat::OVA => write!(f, "OVA"),
            MediaAPIFormat::ONA => write!(f, "ONA"),
            MediaAPIFormat::MUSIC => write!(f, "MUSIC"),
            MediaAPIFormat::MANGA => write!(f, "MANGA"),
            MediaAPIFormat::NOVEL => write!(f, "NOVEL"),
            MediaAPIFormat::OneShot => write!(f, "ONE_SHOT"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum MediaAPIType {
    ANIME,
    MANGA,
}

impl From<String> for MediaAPIType {
    fn from(value: String) -> Self {
        match value.as_str() {
            "ANIME" => MediaAPIType::ANIME,
            "MANGA" => MediaAPIType::MANGA,
            _ => MediaAPIType::ANIME,
        }
    }
}

impl From<MediaAPIType> for String {
    fn from(media_type: MediaAPIType) -> Self {
        match media_type {
            MediaAPIType::ANIME => "ANIME".to_string(),
            MediaAPIType::MANGA => "MANGA".to_string(),
        }
    }
}

impl Display for MediaAPIType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MediaAPIType::ANIME => write!(f, "ANIME"),
            MediaAPIType::MANGA => write!(f, "MANGA"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum MediaAPISeason {
    WINTER,
    SPRING,
    SUMMER,
    FALL,
}

impl From<String> for MediaAPISeason {
    fn from(value: String) -> Self {
        match value.as_str() {
            "WINTER" => MediaAPISeason::WINTER,
            "SPRING" => MediaAPISeason::SPRING,
            "SUMMER" => MediaAPISeason::SUMMER,
            "FALL" => MediaAPISeason::FALL,
            _ => MediaAPISeason::WINTER,
        }
    }
}

impl From<MediaAPISeason> for String {
    fn from(season: MediaAPISeason) -> Self {
        match season {
            MediaAPISeason::WINTER => "WINTER".to_string(),
            MediaAPISeason::SPRING => "SPRING".to_string(),
            MediaAPISeason::SUMMER => "SUMMER".to_string(),
            MediaAPISeason::FALL => "FALL".to_string(),
        }
    }
}

impl Display for MediaAPISeason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MediaAPISeason::WINTER => write!(f, "WINTER"),
            MediaAPISeason::SPRING => write!(f, "SPRING"),
            MediaAPISeason::SUMMER => write!(f, "SUMMER"),
            MediaAPISeason::FALL => write!(f, "FALL"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum MediaAPISort {
    ID,
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
    Favorites,
    FavoritesDesc,
}

impl From<String> for MediaAPISort {
    fn from(value: String) -> Self {
        match value.as_str() {
            "ID" => MediaAPISort::ID,
            "ID_DESC" => MediaAPISort::IdDesc,
            "TITLE_ROMAJI" => MediaAPISort::TitleRomaji,
            "TITLE_ROMAJI_DESC" => MediaAPISort::TitleRomajiDesc,
            "TITLE_ENGLISH" => MediaAPISort::TitleEnglish,
            "TITLE_ENGLISH_DESC" => MediaAPISort::TitleEnglishDesc,
            "TITLE_NATIVE" => MediaAPISort::TitleNative,
            "TITLE_NATIVE_DESC" => MediaAPISort::TitleNativeDesc,
            "TYPE" => MediaAPISort::Type,
            "TYPE_DESC" => MediaAPISort::TypeDesc,
            "FORMAT" => MediaAPISort::Format,
            "FORMAT_DESC" => MediaAPISort::FormatDesc,
            "START_DATE" => MediaAPISort::StartDate,
            "START_DATE_DESC" => MediaAPISort::StartDateDesc,
            "END_DATE" => MediaAPISort::EndDate,
            "END_DATE_DESC" => MediaAPISort::EndDateDesc,
            "SCORE" => MediaAPISort::Score,
            "SCORE_DESC" => MediaAPISort::ScoreDesc,
            "POPULARITY" => MediaAPISort::Popularity,
            "POPULARITY_DESC" => MediaAPISort::PopularityDesc,
            "TRENDING" => MediaAPISort::Trending,
            "TRENDING_DESC" => MediaAPISort::TrendingDesc,
            "EPISODES" => MediaAPISort::Episodes,
            "EPISODES_DESC" => MediaAPISort::EpisodesDesc,
            "DURATION" => MediaAPISort::Duration,
            "DURATION_DESC" => MediaAPISort::DurationDesc,
            "STATUS" => MediaAPISort::Status,
            "STATUS_DESC" => MediaAPISort::StatusDesc,
            "CHAPTERS" => MediaAPISort::Chapters,
            "CHAPTERS_DESC" => MediaAPISort::ChaptersDesc,
            "VOLUMES" => MediaAPISort::Volumes,
            "VOLUMES_DESC" => MediaAPISort::VolumesDesc,
            "UPDATED_AT" => MediaAPISort::UpdatedAt,
            "UPDATED_AT_DESC" => MediaAPISort::UpdatedAtDesc,
            "SEARCH_MATCH" => MediaAPISort::SearchMatch,
            "FAVORITES" => MediaAPISort::Favorites,
            "FAVORITES_DESC" => MediaAPISort::FavoritesDesc,
            _ => MediaAPISort::ID,
        }
    }
}

impl From<MediaAPISort> for String {
    fn from(sort: MediaAPISort) -> Self {
        match sort {
            MediaAPISort::ID => "ID".to_string(),
            MediaAPISort::IdDesc => "ID_DESC".to_string(),
            MediaAPISort::TitleRomaji => "TITLE_ROMAJI".to_string(),
            MediaAPISort::TitleRomajiDesc => "TITLE_ROMAJI_DESC".to_string(),
            MediaAPISort::TitleEnglish => "TITLE_ENGLISH".to_string(),
            MediaAPISort::TitleEnglishDesc => "TITLE_ENGLISH_DESC".to_string(),
            MediaAPISort::TitleNative => "TITLE_NATIVE".to_string(),
            MediaAPISort::TitleNativeDesc => "TITLE_NATIVE_DESC".to_string(),
            MediaAPISort::Type => "TYPE".to_string(),
            MediaAPISort::TypeDesc => "TYPE_DESC".to_string(),
            MediaAPISort::Format => "FORMAT".to_string(),
            MediaAPISort::FormatDesc => "FORMAT_DESC".to_string(),
            MediaAPISort::StartDate => "START_DATE".to_string(),
            MediaAPISort::StartDateDesc => "START_DATE_DESC".to_string(),
            MediaAPISort::EndDate => "END_DATE".to_string(),
            MediaAPISort::EndDateDesc => "END_DATE_DESC".to_string(),
            MediaAPISort::Score => "SCORE".to_string(),
            MediaAPISort::ScoreDesc => "SCORE_DESC".to_string(),
            MediaAPISort::Popularity => "POPULARITY".to_string(),
            MediaAPISort::PopularityDesc => "POPULARITY_DESC".to_string(),
            MediaAPISort::Trending => "TRENDING".to_string(),
            MediaAPISort::TrendingDesc => "TRENDING_DESC".to_string(),
            MediaAPISort::Episodes => "EPISODES".to_string(),
            MediaAPISort::EpisodesDesc => "EPISODES_DESC".to_string(),
            MediaAPISort::Duration => "DURATION".to_string(),
            MediaAPISort::DurationDesc => "DURATION_DESC".to_string(),
            MediaAPISort::Status => "STATUS".to_string(),
            MediaAPISort::StatusDesc => "STATUS_DESC".to_string(),
            MediaAPISort::Chapters => "CHAPTERS".to_string(),
            MediaAPISort::ChaptersDesc => "CHAPTERS_DESC".to_string(),
            MediaAPISort::Volumes => "VOLUMES".to_string(),
            MediaAPISort::VolumesDesc => "VOLUMES_DESC".to_string(),
            MediaAPISort::UpdatedAt => "UPDATED_AT".to_string(),
            MediaAPISort::UpdatedAtDesc => "UPDATED_AT_DESC".to_string(),
            MediaAPISort::SearchMatch => "SEARCH_MATCH".to_string(),
            MediaAPISort::Favorites => "FAVORITES".to_string(),
            MediaAPISort::FavoritesDesc => "FAVORITES_DESC".to_string(),
        }
    }
}

impl Display for MediaAPISort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MediaAPISort::ID => write!(f, "ID"),
            MediaAPISort::IdDesc => write!(f, "ID_DESC"),
            MediaAPISort::TitleRomaji => write!(f, "TITLE_ROMAJI"),
            MediaAPISort::TitleRomajiDesc => write!(f, "TITLE_ROMAJI_DESC"),
            MediaAPISort::TitleEnglish => write!(f, "TITLE_ENGLISH"),
            MediaAPISort::TitleEnglishDesc => write!(f, "TITLE_ENGLISH_DESC"),
            MediaAPISort::TitleNative => write!(f, "TITLE_NATIVE"),
            MediaAPISort::TitleNativeDesc => write!(f, "TITLE_NATIVE_DESC"),
            MediaAPISort::Type => write!(f, "TYPE"),
            MediaAPISort::TypeDesc => write!(f, "TYPE_DESC"),
            MediaAPISort::Format => write!(f, "FORMAT"),
            MediaAPISort::FormatDesc => write!(f, "FORMAT_DESC"),
            MediaAPISort::StartDate => write!(f, "START_DATE"),
            MediaAPISort::StartDateDesc => write!(f, "START_DATE_DESC"),
            MediaAPISort::EndDate => write!(f, "END_DATE"),
            MediaAPISort::EndDateDesc => write!(f, "END_DATE_DESC"),
            MediaAPISort::Score => write!(f, "SCORE"),
            MediaAPISort::ScoreDesc => write!(f, "SCORE_DESC"),
            MediaAPISort::Popularity => write!(f, "POPULARITY"),
            MediaAPISort::PopularityDesc => write!(f, "POPULARITY_DESC"),
            MediaAPISort::Trending => write!(f, "TRENDING"),
            MediaAPISort::TrendingDesc => write!(f, "TRENDING_DESC"),
            MediaAPISort::Episodes => write!(f, "EPISODES"),
            MediaAPISort::EpisodesDesc => write!(f, "EPISODES_DESC"),
            MediaAPISort::Duration => write!(f, "DURATION"),
            MediaAPISort::DurationDesc => write!(f, "DURATION_DESC"),
            MediaAPISort::Status => write!(f, "STATUS"),
            MediaAPISort::StatusDesc => write!(f, "STATUS_DESC"),
            MediaAPISort::Chapters => write!(f, "CHAPTERS"),
            MediaAPISort::ChaptersDesc => write!(f, "CHAPTERS_DESC"),
            MediaAPISort::Volumes => write!(f, "VOLUMES"),
            MediaAPISort::VolumesDesc => write!(f, "VOLUMES_DESC"),
            MediaAPISort::UpdatedAt => write!(f, "UPDATED_AT"),
            MediaAPISort::UpdatedAtDesc => write!(f, "UPDATED_AT_DESC"),
            MediaAPISort::SearchMatch => write!(f, "SEARCH_MATCH"),
            MediaAPISort::Favorites => write!(f, "FAVORITES"),
            MediaAPISort::FavoritesDesc => write!(f, "FAVORITES_DESC"),
        }
    }
}

const QUERY: &str = r#"
  id
  idMal
  title {
    romaji
    english
    native
    userPreferred
  }
  type
  format
  status(version: 2)
  description
  startDate {
    year
    month
    day
  }
  endDate {
    year
    month
    day
  }
  season
  seasonYear
  seasonInt
  episodes
  duration
  chapters
  volumes
  countryOfOrigin
  isLicensed
  source(version: 3)
  hashtag
  trailer {
    id
    site
    thumbnail
  }
  updatedAt
  coverImage {
    extraLarge
    large
    medium
    color
  }
  bannerImage
  genres
  synonyms
  averageScore
  meanScore
  popularity
  isLocked
  trending
  favourites
  tags {
    id
    name
    description
    category
    rank
    isGeneralSpoiler
    isMediaSpoiler
    isAdult
    userId
  }
  isFavouriteBlocked
  isAdult
  nextAiringEpisode {
    id
    airingAt
    timeUntilAiring
    episode
    mediaId
  }
  externalLinks {
    id
    url
    site
    siteId
    type
    language
    color
    icon
    notes
    isDisabled
  }
  streamingEpisodes {
    title
    thumbnail
    url
    site
  }
  rankings {
    id
    rank
    type
    format
    year
    season
    allTime
    context
  }
  stats {
    scoreDistribution {
      score
      amount
    }
    statusDistribution {
      status
      amount
    }
  }
  siteUrl
  autoCreateForumThread
  isRecommendationBlocked
  isReviewBlocked
  modNotes
}
"#;

impl MediaAPIBuilder {
    pub fn new() -> Self {
        Self {
            id: None,
            id_mal: None,
            start_date: None,
            end_date: None,
            season: None,
            season_year: None,
            media_type: None,
            format: None,
            status: None,
            episodes: None,
            duration: None,
            chapters: None,
            volumes: None,
            is_adult: None,
            genre: None,
            tag: None,
            minimum_tag_rank: None,
            tag_category: None,
            licensed_by: None,
            licensed_by_id: None,
            average_score: None,
            popularity: None,
            source: None,
            country_of_origin: None,
            search: None,
            id_not: None,
            id_in: None,
            id_not_in: None,
            id_mal_not: None,
            id_mal_in: None,
            id_mal_not_in: None,
            start_date_greater: None,
            start_date_lesser: None,
            start_date_like: None,
            end_date_greater: None,
            end_date_lesser: None,
            end_date_like: None,
            format_in: None,
            format_not: None,
            format_not_in: None,
            status_in: None,
            status_not: None,
            status_not_in: None,
            episodes_greater: None,
            episodes_lesser: None,
            duration_greater: None,
            duration_lesser: None,
            chapters_greater: None,
            chapters_lesser: None,
            volumes_greater: None,
            volumes_lesser: None,
            genre_in: None,
            genre_not_in: None,
            tag_in: None,
            tag_not_in: None,
            tag_category_in: None,
            tag_category_not_in: None,
            licensed_by_in: None,
            licensed_by_id_in: None,
            average_score_not: None,
            average_score_greater: None,
            average_score_lesser: None,
            popularity_not: None,
            popularity_greater: None,
            popularity_lesser: None,
            source_in: None,
            sort: None,
            query: None,
        }
    }

    pub fn id(mut self, id: i32) -> Self {
        self.id = Some(id);
        self
    }

    pub fn id_mal(mut self, id_mal: i32) -> Self {
        self.id_mal = Some(id_mal);
        self
    }

    pub fn start_date(mut self, start_date: FuzzyDate) -> Self {
        self.start_date = Some(start_date);
        self
    }

    pub fn end_date(mut self, end_date: FuzzyDate) -> Self {
        self.end_date = Some(end_date);
        self
    }

    pub fn season(mut self, season: MediaAPISeason) -> Self {
        self.season = Some(season);
        self
    }

    pub fn season_year(mut self, season_year: i32) -> Self {
        self.season_year = Some(season_year);
        self
    }

    pub fn media_type(mut self, media_type: MediaAPIType) -> Self {
        self.media_type = Some(media_type);
        self
    }

    pub fn format(mut self, format: MediaAPIFormat) -> Self {
        self.format = Some(format);
        self
    }

    pub fn status(mut self, status: MediaAPIStatus) -> Self {
        self.status = Some(status);
        self
    }

    pub fn episodes(mut self, episodes: i32) -> Self {
        self.episodes = Some(episodes);
        self
    }

    pub fn duration(mut self, duration: i32) -> Self {
        self.duration = Some(duration);
        self
    }

    pub fn chapters(mut self, chapters: i32) -> Self {
        self.chapters = Some(chapters);
        self
    }

    pub fn volumes(mut self, volumes: i32) -> Self {
        self.volumes = Some(volumes);
        self
    }

    pub fn is_adult(mut self, is_adult: bool) -> Self {
        self.is_adult = Some(is_adult);
        self
    }

    pub fn genre(mut self, genre: String) -> Self {
        self.genre = Some(genre);
        self
    }

    pub fn tag(mut self, tag: String) -> Self {
        self.tag = Some(tag);
        self
    }

    pub fn minimum_tag_rank(mut self, minimum_tag_rank: i32) -> Self {
        self.minimum_tag_rank = Some(minimum_tag_rank);
        self
    }

    pub fn tag_category(mut self, tag_category: String) -> Self {
        self.tag_category = Some(tag_category);
        self
    }

    pub fn licensed_by(mut self, licensed_by: String) -> Self {
        self.licensed_by = Some(licensed_by);
        self
    }

    pub fn licensed_by_id(mut self, licensed_by_id: i32) -> Self {
        self.licensed_by_id = Some(licensed_by_id);
        self
    }

    pub fn average_score(mut self, average_score: i32) -> Self {
        self.average_score = Some(average_score);
        self
    }

    pub fn popularity(mut self, popularity: i32) -> Self {
        self.popularity = Some(popularity);
        self
    }

    pub fn source(mut self, source: MediaAPISource) -> Self {
        self.source = Some(source);
        self
    }

    pub fn country_of_origin(mut self, country_of_origin: String) -> Self {
        self.country_of_origin = Some(country_of_origin);
        self
    }

    pub fn search(mut self, search: String) -> Self {
        self.search = Some(search);
        self
    }

    pub fn id_not(mut self, id_not: i32) -> Self {
        self.id_not = Some(id_not);
        self
    }

    pub fn id_in(mut self, id_in: Vec<i32>) -> Self {
        self.id_in = Some(id_in);
        self
    }

    pub fn id_not_in(mut self, id_not_in: Vec<i32>) -> Self {
        self.id_not_in = Some(id_not_in);
        self
    }

    pub fn id_mal_not(mut self, id_mal_not: i32) -> Self {
        self.id_mal_not = Some(id_mal_not);
        self
    }

    pub fn id_mal_in(mut self, id_mal_in: Vec<i32>) -> Self {
        self.id_mal_in = Some(id_mal_in);
        self
    }

    pub fn id_mal_not_in(mut self, id_mal_not_in: Vec<i32>) -> Self {
        self.id_mal_not_in = Some(id_mal_not_in);
        self
    }

    pub fn start_date_greater(mut self, start_date_greater: FuzzyDate) -> Self {
        self.start_date_greater = Some(start_date_greater);
        self
    }

    pub fn start_date_lesser(mut self, start_date_lesser: FuzzyDate) -> Self {
        self.start_date_lesser = Some(start_date_lesser);
        self
    }

    pub fn start_date_like(mut self, start_date_like: String) -> Self {
        self.start_date_like = Some(start_date_like);
        self
    }

    pub fn end_date_greater(mut self, end_date_greater: FuzzyDate) -> Self {
        self.end_date_greater = Some(end_date_greater);
        self
    }

    pub fn end_date_lesser(mut self, end_date_lesser: FuzzyDate) -> Self {
        self.end_date_lesser = Some(end_date_lesser);
        self
    }

    pub fn end_date_like(mut self, end_date_like: String) -> Self {
        self.end_date_like = Some(end_date_like);
        self
    }

    pub fn format_in(mut self, format_in: Vec<MediaAPIFormat>) -> Self {
        self.format_in = Some(format_in);
        self
    }

    pub fn format_not(mut self, format_not: MediaAPIFormat) -> Self {
        self.format_not = Some(format_not);
        self
    }

    pub fn format_not_in(mut self, format_not_in: Vec<MediaAPIFormat>) -> Self {
        self.format_not_in = Some(format_not_in);
        self
    }

    pub fn status_in(mut self, status_in: Vec<MediaAPIStatus>) -> Self {
        self.status_in = Some(status_in);
        self
    }

    pub fn status_not(mut self, status_not: MediaAPIStatus) -> Self {
        self.status_not = Some(status_not);
        self
    }

    pub fn status_not_in(mut self, status_not_in: Vec<MediaAPIStatus>) -> Self {
        self.status_not_in = Some(status_not_in);
        self
    }

    pub fn episodes_greater(mut self, episodes_greater: i32) -> Self {
        self.episodes_greater = Some(episodes_greater);
        self
    }

    pub fn episodes_lesser(mut self, episodes_lesser: i32) -> Self {
        self.episodes_lesser = Some(episodes_lesser);
        self
    }

    pub fn duration_greater(mut self, duration_greater: i32) -> Self {
        self.duration_greater = Some(duration_greater);
        self
    }

    pub fn duration_lesser(mut self, duration_lesser: i32) -> Self {
        self.duration_lesser = Some(duration_lesser);
        self
    }

    pub fn chapters_greater(mut self, chapters_greater: i32) -> Self {
        self.chapters_greater = Some(chapters_greater);
        self
    }

    pub fn chapters_lesser(mut self, chapters_lesser: i32) -> Self {
        self.chapters_lesser = Some(chapters_lesser);
        self
    }

    pub fn volumes_greater(mut self, volumes_greater: i32) -> Self {
        self.volumes_greater = Some(volumes_greater);
        self
    }

    pub fn volumes_lesser(mut self, volumes_lesser: i32) -> Self {
        self.volumes_lesser = Some(volumes_lesser);
        self
    }

    pub fn genre_in(mut self, genre_in: Vec<String>) -> Self {
        self.genre_in = Some(genre_in);
        self
    }

    pub fn genre_not_in(mut self, genre_not_in: Vec<String>) -> Self {
        self.genre_not_in = Some(genre_not_in);
        self
    }

    pub fn tag_in(mut self, tag_in: Vec<String>) -> Self {
        self.tag_in = Some(tag_in);
        self
    }

    pub fn tag_not_in(mut self, tag_not_in: Vec<String>) -> Self {
        self.tag_not_in = Some(tag_not_in);
        self
    }

    pub fn tag_category_in(mut self, tag_category_in: Vec<String>) -> Self {
        self.tag_category_in = Some(tag_category_in);
        self
    }

    pub fn tag_category_not_in(mut self, tag_category_not_in: Vec<String>) -> Self {
        self.tag_category_not_in = Some(tag_category_not_in);
        self
    }

    pub fn licensed_by_in(mut self, licensed_by_in: Vec<String>) -> Self {
        self.licensed_by_in = Some(licensed_by_in);
        self
    }

    pub fn licensed_by_id_in(mut self, licensed_by_id_in: Vec<i32>) -> Self {
        self.licensed_by_id_in = Some(licensed_by_id_in);
        self
    }

    pub fn average_score_not(mut self, average_score_not: i32) -> Self {
        self.average_score_not = Some(average_score_not);
        self
    }

    pub fn average_score_greater(mut self, average_score_greater: i32) -> Self {
        self.average_score_greater = Some(average_score_greater);
        self
    }

    pub fn average_score_lesser(mut self, average_score_lesser: i32) -> Self {
        self.average_score_lesser = Some(average_score_lesser);
        self
    }

    pub fn popularity_not(mut self, popularity_not: i32) -> Self {
        self.popularity_not = Some(popularity_not);
        self
    }

    pub fn popularity_greater(mut self, popularity_greater: i32) -> Self {
        self.popularity_greater = Some(popularity_greater);
        self
    }

    pub fn popularity_lesser(mut self, popularity_lesser: i32) -> Self {
        self.popularity_lesser = Some(popularity_lesser);
        self
    }

    pub fn source_in(mut self, source_in: Vec<MediaAPISource>) -> Self {
        self.source_in = Some(source_in);
        self
    }

    pub fn sort(mut self, sort: MediaAPISort) -> Self {
        self.sort = Some(sort);
        self
    }

    pub fn build(mut self, limit: Option<u32>, actual: Option<u32>) -> Self {
        let limit = limit.unwrap_or(1);
        let actual = actual.unwrap_or(0);
        let starting_query = r"query(".to_string();
        let mut media = r"Media(".to_string();
        let mut filter = String::new();
        if let Some(id_filter) = &self.id {
            filter.push_str(format!("$id: Int = {},", id_filter).as_str());
            media.push_str("id: $id,");
        }
        if let Some(id_mal_filter) = &self.id_mal {
            filter.push_str(format!("$idMal: Int = {},", id_mal_filter).as_str());
            media.push_str("idMal: $idMal,");
        }
        if let Some(start_date_filter) = &self.start_date {
            filter.push_str(format!("$startDate: FuzzyDateInt = {},", start_date_filter).as_str());
            media.push_str("startDate: $startDate,");
        }
        if let Some(end_date_filter) = &self.end_date {
            filter.push_str(format!("$endDate: FuzzyDateInt = {},", end_date_filter).as_str());
            media.push_str("endDate: $endDate,");
        }
        if let Some(season_filter) = &self.season {
            filter.push_str(format!("$season: MediaSeason = {},", season_filter).as_str());
            media.push_str("season: $season,");
        }
        if let Some(season_year_filter) = &self.season_year {
            filter.push_str(format!("$seasonYear: Int = {},", season_year_filter).as_str());
            media.push_str("seasonYear: $seasonYear,");
        }
        if let Some(media_type_filter) = &self.media_type {
            filter.push_str(format!("$type: MediaType = {},", media_type_filter).as_str());
            media.push_str("type: $type,");
        }
        if let Some(format_filter) = &self.format {
            filter.push_str(format!("$format: MediaFormat = {},", format_filter).as_str());
            media.push_str("format: $format,");
        }
        if let Some(status_filter) = &self.status {
            filter.push_str(format!("$status: MediaStatus = {},", status_filter).as_str());
            media.push_str("status: $status,");
        }
        if let Some(episodes_filter) = &self.episodes {
            filter.push_str(format!("$episodes: Int = {},", episodes_filter).as_str());
            media.push_str("episodes: $episodes,");
        }
        if let Some(duration_filter) = &self.duration {
            filter.push_str(format!("$duration: Int = {},", duration_filter).as_str());
            media.push_str("duration: $duration,");
        }
        if let Some(chapters_filter) = &self.chapters {
            filter.push_str(format!("$chapters: Int = {},", chapters_filter).as_str());
            media.push_str("chapters: $chapters,");
        }
        if let Some(volumes_filter) = &self.volumes {
            filter.push_str(format!("$volumes: Int = {},", volumes_filter).as_str());
            media.push_str("volumes: $volumes,");
        }
        if let Some(is_adult_filter) = &self.is_adult {
            filter.push_str(format!("$isAdult: Boolean = {},", is_adult_filter).as_str());
            media.push_str("isAdult: $isAdult,");
        }
        if let Some(genre_filter) = &self.genre {
            filter.push_str(format!("$genre: String = {},", genre_filter).as_str());
            media.push_str("genre: $genre,");
        }
        if let Some(tag_filter) = &self.tag {
            filter.push_str(format!("$tag: String = {},", tag_filter).as_str());
            media.push_str("tag: $tag,");
        }
        if let Some(minimum_tag_rank_filter) = &self.minimum_tag_rank {
            filter
                .push_str(format!("$minimumTagRank: Int = {},", minimum_tag_rank_filter).as_str());
            media.push_str("minimumTagRank: $minimumTagRank,");
        }
        if let Some(tag_category_filter) = &self.tag_category {
            filter.push_str(format!("$tagCategory: String = {},", tag_category_filter).as_str());
            media.push_str("tagCategory: $tagCategory,");
        }
        if let Some(licensed_by_filter) = &self.licensed_by {
            filter.push_str(format!("$licensedBy: String = {},", licensed_by_filter).as_str());
            media.push_str("licensedBy: $licensedBy,");
        }
        if let Some(licensed_by_id_filter) = &self.licensed_by_id {
            filter.push_str(format!("$licensedById: Int = {},", licensed_by_id_filter).as_str());
            media.push_str("licensedById: $licensedById,");
        }
        if let Some(average_score_filter) = &self.average_score {
            filter.push_str(format!("$averageScore: Int = {},", average_score_filter).as_str());
            media.push_str("averageScore: $averageScore,");
        }
        if let Some(popularity_filter) = &self.popularity {
            filter.push_str(format!("$popularity: Int = {},", popularity_filter).as_str());
            media.push_str("popularity: $popularity,");
        }
        if let Some(source_filter) = &self.source {
            filter.push_str(format!("$source: MediaSource = {},", source_filter).as_str());
            media.push_str("source: $source,");
        }
        if let Some(country_of_origin_filter) = &self.country_of_origin {
            filter.push_str(
                format!(
                    "$countryOfOrigin: CountryCode = {},",
                    country_of_origin_filter
                )
                .as_str(),
            );
            media.push_str("countryOfOrigin: $countryOfOrigin,");
        }
        if let Some(search_filter) = &self.search {
            filter.push_str(format!("$search: String = {},", search_filter).as_str());
            media.push_str("search: $search,");
        }
        if let Some(id_not_filter) = &self.id_not {
            filter.push_str(format!("$idNot: Int = {},", id_not_filter).as_str());
            media.push_str("id_not: $idNot,");
        }
        if let Some(id_in_filter) = &self.id_in {
            let id_in = id_in_filter
                .iter()
                .map(|id| id.to_string())
                .collect::<Vec<String>>()
                .join(", ");
            let id_in_filter = format!("[{}]", id_in);
            filter.push_str(format!("$idIn: [Int] = {},", id_in_filter).as_str());
            media.push_str("id_in: $idIn,");
        }
        if let Some(id_not_in_filter) = &self.id_not_in {
            let id_in = id_not_in_filter
                .iter()
                .map(|id| id.to_string())
                .collect::<Vec<String>>()
                .join(", ");
            let id_not_in_filter = format!("[{}]", id_in);
            filter.push_str(format!("$idNotIn: [Int] = {},", id_not_in_filter).as_str());
            media.push_str("id_not_in: $idNotIn,");
        }
        if let Some(id_mal_not_filter) = &self.id_mal_not {
            filter.push_str(format!("$idMalNot: Int = {},", id_mal_not_filter).as_str());
            media.push_str("idMal_not: $idMalNot,");
        }
        if let Some(id_mal_in_filter) = &self.id_mal_in {
            let id_in = id_mal_in_filter
                .iter()
                .map(|id| id.to_string())
                .collect::<Vec<String>>()
                .join(", ");
            let id_mal_in_filter = format!("[{}]", id_in);
            filter.push_str(format!("$idMalIn: [Int] = {},", id_mal_in_filter).as_str());
            media.push_str("idMal_in: $idMalIn,");
        }
        if let Some(id_mal_not_in_filter) = &self.id_mal_not_in {
            let id_in = id_mal_not_in_filter
                .iter()
                .map(|id| id.to_string())
                .collect::<Vec<String>>()
                .join(", ");
            let id_mal_not_in_filter = format!("[{}]", id_in);
            filter.push_str(format!("$idMalNotIn: [Int] = {},", id_mal_not_in_filter).as_str());
            media.push_str("idMal_not_in: $idMalNotIn,");
        }
        if let Some(start_date_greater_filter) = &self.start_date_greater {
            filter.push_str(
                format!(
                    "$startDateGreater: FuzzyDateInt = {},",
                    start_date_greater_filter
                )
                .as_str(),
            );
            media.push_str("startDate_greater: $startDateGreater,");
        }
        if let Some(start_date_lesser_filter) = &self.start_date_lesser {
            filter.push_str(
                format!(
                    "$startDateLesser: FuzzyDateInt = {},",
                    start_date_lesser_filter
                )
                .as_str(),
            );
            media.push_str("startDate_lesser: $startDateLesser,");
        }
        if let Some(start_date_like_filter) = &self.start_date_like {
            filter
                .push_str(format!("$startDateLike: String = {},", start_date_like_filter).as_str());
            media.push_str("startDate_like: $startDateLike,");
        }
        if let Some(end_date_greater_filter) = &self.end_date_greater {
            filter.push_str(
                format!(
                    "$endDateGreater: FuzzyDateInt = {},",
                    end_date_greater_filter
                )
                .as_str(),
            );
            media.push_str("endDate_greater: $endDateGreater,");
        }
        if let Some(end_date_lesser_filter) = &self.end_date_lesser {
            filter.push_str(
                format!("$endDateLesser: FuzzyDateInt = {},", end_date_lesser_filter).as_str(),
            );
            media.push_str("endDate_lesser: $endDateLesser,");
        }
        if let Some(end_date_like_filter) = &self.end_date_like {
            filter.push_str(format!("$endDateLike: String = {},", end_date_like_filter).as_str());
            media.push_str("endDate_like: $endDateLike,");
        }
        if let Some(format_in_filter) = &self.format_in {
            let format_in = format_in_filter
                .iter()
                .map(|format| format.to_string())
                .collect::<Vec<String>>()
                .join(", ");
            let format_in_filter = format!("[{}]", format_in);
            filter.push_str(format!("$formatIn: [MediaFormat] = {},", format_in_filter).as_str());
            media.push_str("format_in: $formatIn,");
        }
        if let Some(format_not_filter) = &self.format_not {
            filter.push_str(format!("$formatNot: MediaFormat = {},", format_not_filter).as_str());
            media.push_str("format_not: $formatNot,");
        }
        if let Some(format_not_in_filter) = &self.format_not_in {
            let format_in = format_not_in_filter
                .iter()
                .map(|format| format.to_string())
                .collect::<Vec<String>>()
                .join(", ");
            let format_not_in_filter = format!("[{}]", format_in);
            filter.push_str(
                format!("$formatNotIn: [MediaFormat] = {},", format_not_in_filter).as_str(),
            );
            media.push_str("format_not_in: $formatNotIn,");
        }
        if let Some(status_in_filter) = &self.status_in {
            let status_in = status_in_filter
                .iter()
                .map(|status| status.to_string())
                .collect::<Vec<String>>()
                .join(", ");
            let status_in_filter = format!("[{}]", status_in);
            filter.push_str(format!("$statusIn: [MediaStatus] = {},", status_in_filter).as_str());
            media.push_str("status_in: $statusIn,");
        }
        if let Some(status_not_filter) = &self.status_not {
            filter.push_str(format!("$statusNot: MediaStatus = {},", status_not_filter).as_str());
            media.push_str("status_not: $statusNot,");
        }
        if let Some(status_not_in_filter) = &self.status_not_in {
            let status_in = status_not_in_filter
                .iter()
                .map(|status| status.to_string())
                .collect::<Vec<String>>()
                .join(", ");
            let status_not_in_filter = format!("[{}]", status_in);
            filter.push_str(
                format!("$statusNotIn: [MediaStatus] = {},", status_not_in_filter).as_str(),
            );
            media.push_str("status_not_in: $statusNotIn,");
        }
        if let Some(episodes_greater_filter) = &self.episodes_greater {
            filter
                .push_str(format!("$episodesGreater: Int = {},", episodes_greater_filter).as_str());
            media.push_str("episodes_greater: $episodesGreater,");
        }
        if let Some(episodes_lesser_filter) = &self.episodes_lesser {
            filter.push_str(format!("$episodesLesser: Int = {},", episodes_lesser_filter).as_str());
            media.push_str("episodes_lesser: $episodesLesser,");
        }
        if let Some(duration_greater_filter) = &self.duration_greater {
            filter
                .push_str(format!("$durationGreater: Int = {},", duration_greater_filter).as_str());
            media.push_str("duration_greater: $durationGreater,");
        }
        if let Some(duration_lesser_filter) = &self.duration_lesser {
            filter.push_str(format!("$durationLesser: Int = {},", duration_lesser_filter).as_str());
            media.push_str("duration_lesser: $durationLesser,");
        }
        if let Some(chapters_greater_filter) = &self.chapters_greater {
            filter
                .push_str(format!("$chaptersGreater: Int = {},", chapters_greater_filter).as_str());
            media.push_str("chapters_greater: $chaptersGreater,");
        }
        if let Some(chapters_lesser_filter) = &self.chapters_lesser {
            filter.push_str(format!("$chaptersLesser: Int = {},", chapters_lesser_filter).as_str());
            media.push_str("chapters_lesser: $chaptersLesser,");
        }
        if let Some(volumes_greater_filter) = &self.volumes_greater {
            filter.push_str(format!("$volumesGreater: Int = {},", volumes_greater_filter).as_str());
            media.push_str("volumes_greater: $volumesGreater,");
        }
        if let Some(volumes_lesser_filter) = &self.volumes_lesser {
            filter.push_str(format!("$volumesLesser: Int = {},", volumes_lesser_filter).as_str());
            media.push_str("volumes_lesser: $volumesLesser,");
        }
        if let Some(genre_in_filter) = &self.genre_in {
            let genre_in = genre_in_filter
                .iter()
                .map(|genre| genre.to_string())
                .collect::<Vec<String>>()
                .join(", ");
            let genre_in_filter = format!("[{}]", genre_in);
            filter.push_str(format!("$genreIn: [String] = {},", genre_in_filter).as_str());
            media.push_str("genre_in: $genreIn,");
        }
        if let Some(genre_not_in_filter) = &self.genre_not_in {
            let genre_in = genre_not_in_filter
                .iter()
                .map(|genre| genre.to_string())
                .collect::<Vec<String>>()
                .join(", ");
            let genre_not_in_filter = format!("[{}]", genre_in);
            filter.push_str(format!("$genreNotIn: [String] = {},", genre_not_in_filter).as_str());
            media.push_str("genre_not_in: $genreNotIn,");
        }
        if let Some(tag_in_filter) = &self.tag_in {
            let tag_in = tag_in_filter
                .iter()
                .map(|tag| tag.to_string())
                .collect::<Vec<String>>()
                .join(", ");
            let tag_in_filter = format!("[{}]", tag_in);
            filter.push_str(format!("$tagIn: [String] = {},", tag_in_filter).as_str());
            media.push_str("tag_in: $tagIn,");
        }
        if let Some(tag_not_in_filter) = &self.tag_not_in {
            let tag_in = tag_not_in_filter
                .iter()
                .map(|tag| tag.to_string())
                .collect::<Vec<String>>()
                .join(", ");
            let tag_not_in_filter = format!("[{}]", tag_in);
            filter.push_str(format!("$tagNotIn: [String] = {},", tag_not_in_filter).as_str());
            media.push_str("tag_not_in: $tagNotIn,");
        }
        if let Some(tag_category_in_filter) = &self.tag_category_in {
            let tag_category_in = tag_category_in_filter
                .iter()
                .map(|tag_category| tag_category.to_string())
                .collect::<Vec<String>>()
                .join(", ");
            let tag_category_in_filter = format!("[{}]", tag_category_in);
            filter.push_str(
                format!("$tagCategoryIn: [String] = {},", tag_category_in_filter).as_str(),
            );
            media.push_str("tag_category_in: $tagCategoryIn,");
        }
        if let Some(tag_category_not_in_filter) = &self.tag_category_not_in {
            let tag_category_in = tag_category_not_in_filter
                .iter()
                .map(|tag_category| tag_category.to_string())
                .collect::<Vec<String>>()
                .join(", ");
            let tag_category_not_in_filter = format!("[{}]", tag_category_in);
            filter.push_str(
                format!(
                    "$tagCategoryNotIn: [String] = {},",
                    tag_category_not_in_filter
                )
                .as_str(),
            );
            media.push_str("tag_category_not_in: $tagCategoryNotIn,");
        }
        if let Some(licensed_by_in_filter) = &self.licensed_by_in {
            let licensed_by_in = licensed_by_in_filter
                .iter()
                .map(|licensed_by| licensed_by.to_string())
                .collect::<Vec<String>>()
                .join(", ");
            let licensed_by_in_filter = format!("[{}]", licensed_by_in);
            filter
                .push_str(format!("$licensedByIn: [String] = {},", licensed_by_in_filter).as_str());
            media.push_str("licensed_by_in: $licensedByIn,");
        }
        if let Some(licensed_by_id_in_filter) = &self.licensed_by_id_in {
            let licensed_by_id_in = licensed_by_id_in_filter
                .iter()
                .map(|licensed_by_id| licensed_by_id.to_string())
                .collect::<Vec<String>>()
                .join(", ");
            let licensed_by_id_in_filter = format!("[{}]", licensed_by_id_in);
            filter.push_str(
                format!("$licensedByIdIn: [Int] = {},", licensed_by_id_in_filter).as_str(),
            );
            media.push_str("licensed_by_id_in: $licensedByIdIn,");
        }
        if let Some(average_score_not_filter) = &self.average_score_not {
            filter.push_str(
                format!("$averageScoreNot: Int = {},", average_score_not_filter).as_str(),
            );
            media.push_str("average_score_not: $averageScoreNot,");
        }
        if let Some(average_score_greater_filter) = &self.average_score_greater {
            filter.push_str(
                format!(
                    "$averageScoreGreater: Int = {},",
                    average_score_greater_filter
                )
                .as_str(),
            );
            media.push_str("average_score_greater: $averageScoreGreater,");
        }
        if let Some(average_score_lesser_filter) = &self.average_score_lesser {
            filter.push_str(
                format!(
                    "$averageScoreLesser: Int = {},",
                    average_score_lesser_filter
                )
                .as_str(),
            );
            media.push_str("average_score_lesser: $averageScoreLesser,");
        }
        if let Some(popularity_not_filter) = &self.popularity_not {
            filter.push_str(format!("$popularityNot: Int = {},", popularity_not_filter).as_str());
            media.push_str("popularity_not: $popularityNot,");
        }
        if let Some(popularity_greater_filter) = &self.popularity_greater {
            filter.push_str(
                format!("$popularityGreater: Int = {},", popularity_greater_filter).as_str(),
            );
            media.push_str("popularity_greater: $popularityGreater,");
        }
        if let Some(popularity_lesser_filter) = &self.popularity_lesser {
            filter.push_str(
                format!("$popularityLesser: Int = {},", popularity_lesser_filter).as_str(),
            );
            media.push_str("popularity_lesser: $popularityLesser,");
        }
        if let Some(source_in_filter) = &self.source_in {
            let source_in = source_in_filter
                .iter()
                .map(|source| source.to_string())
                .collect::<Vec<String>>()
                .join(", ");
            let source_in_filter = format!("[{}]", source_in);
            filter.push_str(format!("$sourceIn: [MediaSource] = {},", source_in_filter).as_str());
            media.push_str("source_in: $sourceIn,");
        }
        if let Some(sort_filter) = &self.sort {
            filter.push_str(format!("$sort: MediaSort = {},", sort_filter).as_str());
            media.push_str("sort: $sort,");
        }
        if let Some(query_filter) = &self.query {
            filter.push_str(format!("$query: String = {},", query_filter).as_str());
            media.push_str("query: $query,");
        }
        let end_query = r"}";
        let start_query = r"{";
        let query = format!(
            "{}{}){}{}){}{}{}",
            starting_query, filter, start_query, media, start_query, QUERY, end_query
        );

        self.query = Some(query);
        self
    }

    pub fn get_query(self) -> Option<String> {
        self.query
    }
}
