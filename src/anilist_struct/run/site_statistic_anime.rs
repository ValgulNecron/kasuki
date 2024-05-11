use serde::Deserialize;
use serde_json::json;

use crate::helper::make_anilist_cached_request::make_request_anilist;
use crate::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};

#[derive(Debug, Deserialize, Clone)]
pub struct SiteStatisticsAnimeWrapper {
    pub data: SiteStatisticsAnimeData,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SiteStatisticsAnimeData {
    #[serde(rename = "SiteStatistics")]
    pub site_statistics: SiteStatisticsAnimeContainer,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SiteStatisticsAnimeContainer {
    pub anime: SiteStatisticAnime,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SiteStatisticAnime {
    #[serde(rename = "pageInfo")]
    pub page_info: SiteStatisticsAnimePageInfo,
    pub nodes: Vec<SiteStatisticsAnimeNode>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SiteStatisticsAnimePageInfo {
    #[serde(rename = "currentPage")]
    pub current_page: i32,
    #[serde(rename = "lastPage")]
    pub last_page: i32,
    pub total: i32,
    #[serde(rename = "hasNextPage")]
    pub has_next_page: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SiteStatisticsAnimeNode {
    pub date: i64,
    pub count: i32,
    pub change: i32,
}

/// `SiteStatisticsAnimeWrapper` is an implementation block for the `SiteStatisticsAnimeWrapper` struct.
impl SiteStatisticsAnimeWrapper {
    /// `new_anime` is an asynchronous function that creates a new anime site statistics.
    /// It takes a `page_number` as a parameter.
    /// `page_number` is a 64-bit integer that represents the page number.
    /// It returns a `Result` that contains a tuple of `SiteStatisticsAnimeWrapper` and a `String`, or an `AppError`.
    ///
    /// This function first defines a GraphQL query string that takes a `page_number` as a variable.
    /// It then creates a JSON object with the query string and the variable.
    /// The `page_number` variable is set to the `page_number` parameter.
    /// It makes a request to AniList with the JSON object and waits for the response.
    /// It then deserializes the response into a `SiteStatisticsAnimeWrapper` and returns it along with the response string.
    ///
    /// # Arguments
    ///
    /// * `page_number` - A 64-bit integer that represents the page number.
    ///
    /// # Returns
    ///
    /// * `Result<(SiteStatisticsAnimeWrapper, String), AppError>` - A Result that contains a tuple of `SiteStatisticsAnimeWrapper` and a `String`, or an `AppError`.
    pub async fn new_anime(
        page_number: i64,
    ) -> Result<(SiteStatisticsAnimeWrapper, String), AppError> {
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
        let api_response: SiteStatisticsAnimeWrapper = serde_json::from_str(&res).map_err(|e| {
            AppError::new(
                format!("Error getting the anime page {}. {}", page_number, e),
                ErrorType::WebRequest,
                ErrorResponseType::Followup,
            )
        })?;
        Ok((api_response, res))
    }

    /// `has_next_page` is a function that checks if there is a next page.
    /// It takes no parameters.
    /// It returns a boolean that represents if there is a next page.
    ///
    /// This function gets the `has_next_page` field from the `page_info` of the `anime` in the `site_statistics` of the `data` and returns it.
    ///
    /// # Returns
    ///
    /// * `bool` - A boolean that represents if there is a next page.
    pub fn has_next_page(&self) -> bool {
        self.data.site_statistics.anime.page_info.has_next_page
    }
}
