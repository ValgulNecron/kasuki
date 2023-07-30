use serde::Deserialize;
use serde_json::json;

use crate::cmd::general_module::html_parser::convert_to_markdown;
use crate::cmd::general_module::lang_struct::{AnimeLocalisedText, MediaLocalisedText};
use crate::cmd::general_module::request::make_request_anilist;
use crate::cmd::general_module::trim::trim;

#[derive(Debug, Deserialize)]
pub struct MediaWrapper {
    pub data: MediaData,
}

#[derive(Debug, Deserialize)]
pub struct MediaData {
    #[serde(rename = "Media")]
    pub media: Media,
}

#[derive(Debug, Deserialize)]
pub struct Media {
    pub id: i64,
    pub description: Option<String>,
    pub title: Title,
    pub r#type: Option<String>,
    pub format: Option<String>,
    pub source: Option<String>,
    #[serde(rename = "isAdult")]
    pub is_adult: bool,
    #[serde(rename = "startDate")]
    pub start_date: StartEndDate,
    #[serde(rename = "endDate")]
    pub end_date: StartEndDate,
    pub chapters: Option<i32>,
    pub volumes: Option<i32>,
    pub status: Option<String>,
    pub season: Option<String>,
    #[serde(rename = "isLicensed")]
    pub is_licensed: bool,
    #[serde(rename = "coverImage")]
    pub cover_image: CoverImage,
    #[serde(rename = "bannerImage")]
    pub banner_image: Option<String>,
    pub genres: Vec<Option<String>>,
    pub tags: Vec<Tag>,
    #[serde(rename = "averageScore")]
    pub average_score: Option<i32>,
    #[serde(rename = "meanScore")]
    pub mean_score: Option<i32>,
    pub popularity: Option<i32>,
    pub favourites: Option<i32>,
    #[serde(rename = "siteUrl")]
    pub site_url: Option<String>,
    pub staff: Staff,
}

#[derive(Debug, Deserialize)]
pub struct Title {
    pub romaji: Option<String>,
    pub english: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct StartEndDate {
    pub year: Option<i32>,
    pub month: Option<i32>,
    pub day: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct CoverImage {
    #[serde(rename = "extraLarge")]
    pub extra_large: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Tag {
    pub name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Staff {
    pub edges: Vec<Edge>,
}

#[derive(Debug, Deserialize)]
pub struct Edge {
    pub node: Node,
    pub id: Option<u32>,
    pub role: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Node {
    pub id: Option<u32>,
    pub name: Name,
}

#[derive(Debug, Deserialize)]
pub struct Name {
    pub full: Option<String>,
    #[serde(rename = "userPreferred")]
    pub user_preferred: Option<String>,
}

impl MediaWrapper {
    pub async fn new_anime_by_id(
        search: String,
        localised_text: AnimeLocalisedText,
    ) -> Result<MediaWrapper, String> {
        let query_id: &str = "
    query ($search: Int, $limit: Int = 5) {
		Media (id: $search, type: ANIME){
    id
      description
    title{
      romaji
      english
    }
    type
    format
    source
    isAdult
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
    chapters
    volumes
    status
    season
    isLicensed
    coverImage {
      extraLarge
    }
    bannerImage
    genres
    tags {
      name
    }
    averageScore
    meanScore
    popularity
    favourites
    siteUrl
    staff(perPage: $limit) {
      edges {
        node {
          id
          name {
            full
            userPreferred
          }
        }
        id
        role
      }
    }
  }
}
";

        let json = json!({"query": query_id, "variables": {"search": search}});
        let resp = make_request_anilist(json, false).await;
        // Get json
        let data: MediaWrapper = match serde_json::from_str(&resp) {
            Ok(data) => data,
            Err(error) => {
                println!("Error: {}", error);
                return Err(localised_text.error_anime_not_found.clone());
            }
        };
        return Ok(data);
    }

    pub async fn new_anime_by_search(
        search: &String,
        localised_text: AnimeLocalisedText,
    ) -> Result<MediaWrapper, String> {
        let query_string: &str = "
    query ($search: String, $limit: Int = 5) {
		Media (search: $search, type: ANIME){
    id
      description
    title{
      romaji
      english
    }
    type
    format
    source
    isAdult
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
    chapters
    volumes
    status
    season
    isLicensed
    coverImage {
      extraLarge
    }
    bannerImage
    genres
    tags {
      name
    }
    averageScore
    meanScore
    popularity
    favourites
    siteUrl
    staff(perPage: $limit) {
      edges {
        node {
          id
          name {
            full
            userPreferred
          }
        }
        id
        role
      }
    }
  }
}
";
        let json = json!({"query": query_string, "variables": {"search": search}});
        let resp = make_request_anilist(json, false).await;
        // Get json
        let data: MediaWrapper = match serde_json::from_str(&resp) {
            Ok(data) => data,
            Err(error) => {
                println!("Error: {}", error);
                return Err(localised_text.error_anime_not_found.clone());
            }
        };
        return Ok(data);
    }

    pub async fn new_manga_by_id(
        search: String,
        localised_text: MediaLocalisedText,
    ) -> Result<MediaWrapper, String> {
        let query_id: &str = "
    query ($search: Int, $limit: Int = 5, $format: MediaFormat = NOVEL) {
		Media (id: $search, type: MANGA, format_not: $format){
    id
      description
    title{
      romaji
      english
    }
    type
    format
    source
    isAdult
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
    chapters
    volumes
    status
    season
    isLicensed
    coverImage {
      extraLarge
    }
    bannerImage
    genres
    tags {
      name
    }
    averageScore
    meanScore
    popularity
    favourites
    siteUrl
    staff(perPage: $limit) {
      edges {
        node {
          id
          name {
            full
            userPreferred
          }
        }
        id
        role
      }
    }
  }
}
";

        let json = json!({"query": query_id, "variables": {"search": search}});
        let resp = make_request_anilist(json, false).await;
        // Get json
        let data: MediaWrapper = match serde_json::from_str(&resp) {
            Ok(data) => data,
            Err(error) => {
                println!("Error: {}", error);
                return Err(localised_text.error_no_media.clone());
            }
        };
        return Ok(data);
    }

    pub async fn new_manga_by_search(
        search: String,
        localised_text: MediaLocalisedText,
    ) -> Result<MediaWrapper, String> {
        let query_string: &str = "
    query ($search: String, $limit: Int = 5, $format: MediaFormat = NOVEL) {
		Media (search: $search, type: MANGA, format_not: $format){
    id
      description
    title{
      romaji
      english
    }
    type
    format
    source
    isAdult
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
    chapters
    volumes
    status
    season
    isLicensed
    coverImage {
      extraLarge
    }
    bannerImage
    genres
    tags {
      name
    }
    averageScore
    meanScore
    popularity
    favourites
    siteUrl
    staff(perPage: $limit) {
      edges {
        node {
          id
          name {
            full
            userPreferred
          }
        }
        id
        role
      }
    }
  }
}
";
        let json = json!({"query": query_string, "variables": {"search": search}});
        let resp = make_request_anilist(json, false).await;
        // Get json
        let data: MediaWrapper = match serde_json::from_str(&resp) {
            Ok(data) => data,
            Err(error) => {
                println!("Error: {}", error);
                return Err(localised_text.error_no_media.clone());
            }
        };
        return Ok(data);
    }

    pub async fn new_ln_by_id(
        search: String,
        localised_text: MediaLocalisedText,
    ) -> Result<MediaWrapper, String> {
        let query_id: &str = "
    query ($search: Int, $limit: Int = 5, $format: MediaFormat = NOVEL) {
		Media (id: $search, type: MANGA, format: $format){
    id
      description
    title{
      romaji
      english
    }
    type
    format
    source
    isAdult
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
    chapters
    volumes
    status
    season
    isLicensed
    coverImage {
      extraLarge
    }
    bannerImage
    genres
    tags {
      name
    }
    averageScore
    meanScore
    popularity
    favourites
    siteUrl
    staff(perPage: $limit) {
      edges {
        node {
          id
          name {
            full
            userPreferred
          }
        }
        id
        role
      }
    }
  }
}
";

        let json = json!({"query": query_id, "variables": {"search": search}});
        let resp = make_request_anilist(json, false).await;
        // Get json
        let data: MediaWrapper = match serde_json::from_str(&resp) {
            Ok(data) => data,
            Err(error) => {
                println!("Error: {}", error);
                return Err(localised_text.error_no_media.clone());
            }
        };
        return Ok(data);
    }

    pub async fn new_ln_by_search(
        search: String,
        localised_text: MediaLocalisedText,
    ) -> Result<MediaWrapper, String> {
        let query_string: &str = "
    query ($search: String, $limit: Int = 5, $format: MediaFormat = NOVEL) {
		Media (search: $search, type: MANGA, format: $format){
    id
      description
    title{
      romaji
      english
    }
    type
    format
    source
    isAdult
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
    chapters
    volumes
    status
    season
    isLicensed
    coverImage {
      extraLarge
    }
    bannerImage
    genres
    tags {
      name
    }
    averageScore
    meanScore
    popularity
    favourites
    siteUrl
    staff(perPage: $limit) {
      edges {
        node {
          id
          name {
            full
            userPreferred
          }
        }
        id
        role
      }
    }
  }
}
";

        let json = json!({"query": query_string, "variables": {"search": search}});
        let resp = make_request_anilist(json, false).await;
        // Get json
        let data: MediaWrapper = match serde_json::from_str(&resp) {
            Ok(data) => data,
            Err(error) => {
                println!("Error: {}", error);
                return Err(localised_text.error_no_media.clone());
            }
        };
        return Ok(data);
    }

    pub fn get_nsfw(&self) -> bool {
        self.data.media.is_adult
    }

    pub fn get_banner(&self) -> String {
        format!("https://img.anili.st/media/{}", self.data.media.id)
    }

    pub fn get_desc(&self) -> String {
        let mut desc = self
            .data
            .media
            .description
            .clone()
            .unwrap_or_else(|| "NA".to_string());
        desc = convert_to_markdown(desc);
        let lenght_diff = 4096 - desc.len() as i32;
        if lenght_diff <= 0 {
            desc = trim(desc, lenght_diff)
        }
        desc
    }

    pub fn get_en_title(&self) -> String {
        self.data
            .media
            .title
            .english
            .clone()
            .unwrap_or_else(|| "NA".to_string())
    }

    pub fn get_rj_title(&self) -> String {
        self.data
            .media
            .title
            .romaji
            .clone()
            .unwrap_or_else(|| "NA".to_string())
    }

    pub fn get_thumbnail(&self) -> String {
        self.data.media.cover_image.extra_large.clone().unwrap_or_else(|| "https://imgs.search.brave.com/CYnhSvdQcm9aZe3wG84YY0B19zT2wlAuAkiAGu0mcLc/rs:fit:640:400:1/g:ce/aHR0cDovL3d3dy5m/cmVtb250Z3VyZHdh/cmEub3JnL3dwLWNv/bnRlbnQvdXBsb2Fk/cy8yMDIwLzA2L25v/LWltYWdlLWljb24t/Mi5wbmc".to_string())
    }

    pub fn get_url(&self) -> String {
        self.data
            .media
            .site_url
            .clone()
            .unwrap_or_else(|| "https://example.com".to_string())
    }

    pub fn get_name(&self) -> String {
        format!("{} / {}", self.get_en_title(), self.get_rj_title())
    }

    pub fn get_format(&self) -> String {
        self.data
            .media
            .format
            .clone()
            .unwrap_or_else(|| "N/A".to_string())
    }

    pub fn get_source(&self) -> String {
        self.data
            .media
            .source
            .clone()
            .unwrap_or_else(|| "N/A".to_string())
    }

    pub fn get_start_date(&self) -> String {
        let start_y = self.data.media.start_date.year.unwrap_or_else(|| 0);
        let start_d = self.data.media.start_date.day.unwrap_or_else(|| 0);
        let start_m = self.data.media.start_date.month.unwrap_or_else(|| 0);
        let start_date = if start_y == 0 && start_d == 0 && start_m == 0 {
            "N/A".to_string()
        } else {
            format!("{}/{}/{}", start_d, start_m, start_y)
        };
        start_date
    }

    pub fn get_end_date(&self) -> String {
        let end_y = self.data.media.end_date.year.unwrap_or_else(|| 0);
        let end_d = self.data.media.end_date.day.unwrap_or_else(|| 0);
        let end_m = self.data.media.end_date.month.unwrap_or_else(|| 0);
        let end_date = if end_y == 0 && end_d == 0 && end_m == 0 {
            "N/A".to_string()
        } else {
            format!("{}/{}/{}", end_y, end_d, end_m)
        };
        end_date
    }

    pub fn get_anime_staff(&self, localised_text: AnimeLocalisedText) -> String {
        let mut staff = "".to_string();
        let staffs = &self.data.media.staff.edges;
        for s in staffs {
            let full = s
                .node
                .name
                .full
                .clone()
                .unwrap_or_else(|| "N/A".to_string());
            let user = s
                .node
                .name
                .user_preferred
                .clone()
                .unwrap_or_else(|| "N/A".to_string());
            let role = s.role.clone().unwrap_or_else(|| "N/A".to_string());
            staff.push_str(&format!(
                "{}{}{}{}{}{} | \n",
                &localised_text.desc_part_5,
                full,
                localised_text.desc_part_6,
                user,
                localised_text.desc_part_7,
                role
            ));
        }
        staff
    }

    pub fn get_media_staff(&self, localised_text: MediaLocalisedText) -> String {
        let mut staff = "".to_string();
            let staffs = &self.data.media.staff.edges;
            for s in staffs {
                let full = s.node.name.full.clone().unwrap_or_else(|| "N/A".to_string());
                let user = s
                    .node
                    .name
                    .user_preferred
                    .clone()
                    .unwrap_or_else(|| "N/A".to_string());
                let role = s.role.clone().unwrap_or_else(|| "N/A".to_string());
                staff.push_str(&format!(
                    "{}{}{}{}{}{}\n",
                    &localised_text.full_name,
                    full,
                    &localised_text.user_pref,
                    user,
                    &localised_text.role,
                    role
                ));
            }
        staff
    }

    pub fn get_anime_info(&self, localised_text: AnimeLocalisedText) -> String {
        let format = self.get_format();
        let source = self.get_source();
        let start_date = self.get_start_date();
        let end_date = self.get_end_date();
        let staff = self.get_anime_staff(localised_text.clone());

        format!(
            "{}{}{}{}{}{}{}{} \n {}",
            &localised_text.desc_part_1,
            format,
            &localised_text.desc_part_2,
            source,
            &localised_text.desc_part_3,
            start_date,
            &localised_text.desc_part_4,
            end_date,
            staff
        )
    }

    pub fn get_media_info(&self, localised_text: MediaLocalisedText) -> String {
        let format = self.get_format();
            let source = self.get_source();
            let start_date = self.get_start_date();
            let end_date = self.get_end_date();
            let staff = self.get_media_staff(localised_text.clone());
        format!(
                "{}{}{}{}{}{}{}{} \n {}",
                &localised_text.format,
                format,
                &localised_text.source,
                source,
                &localised_text.start_date,
                start_date,
                &localised_text.end_date,
                end_date,
                staff
            )
    }

    pub fn get_genres(&self) -> String {
        let mut genre = "".to_string();
        let genre_list = &self.data.media.genres;
        for g in genre_list {
            genre += &g.clone().unwrap_or_else(|| "N/A".to_string());
            genre += "\n"
        }
        genre
    }

    pub fn get_tags(&self) -> String {
        let mut tag = "".to_string();
        let tag_list = &self.data.media.tags;
        for t in tag_list.iter().take(5) {
            let tag_name: String = t.name.as_ref().map_or("N/A".to_string(), |s| s.to_string());
            tag += &tag_name;
            tag += "\n";
        }
        tag
    }
}
