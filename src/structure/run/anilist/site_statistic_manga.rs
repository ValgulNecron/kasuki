use serde::Deserialize;
use serde_json::json;

use crate::helper::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use crate::helper::make_anilist_cached_request::make_request_anilist;

#[derive(Debug, Deserialize, Clone)]
pub struct SiteStatisticsMangaWrapper {
    pub data: SiteStatisticsMangaData,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SiteStatisticsMangaData {
    #[serde(rename = "SiteStatistics")]
    pub site_statistics: SiteStatisticsMangaContainer,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SiteStatisticsMangaContainer {
    pub manga: SiteStatisticManga,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SiteStatisticManga {
    #[serde(rename = "pageInfo")]
    pub page_info: SiteStatisticsMangaPageInfo,
    pub nodes: Vec<SiteStatisticsMangaNode>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SiteStatisticsMangaPageInfo {
    #[serde(rename = "currentPage")]
    pub current_page: i32,
    #[serde(rename = "lastPage")]
    pub last_page: i32,
    pub total: i32,
    #[serde(rename = "hasNextPage")]
    pub has_next_page: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SiteStatisticsMangaNode {
    pub date: i64,
    pub count: i32,
    pub change: i32,
}

/// `SiteStatisticsMangaWrapper` is an implementation block for the `SiteStatisticsMangaWrapper` struct.
impl SiteStatisticsMangaWrapper {
    /// `new_manga` is an asynchronous function that creates a new manga site statistics.
    /// It takes a `page_number` as a parameter.
    /// `page_number` is a 64-bit integer that represents the page number.
    /// It returns a `Result` that contains a tuple of `SiteStatisticsMangaWrapper` and a `String`, or an `AppError`.
    ///
    /// This function first defines a GraphQL query string that takes a `page_number` as a variable.
    /// It then creates a JSON object with the query string and the variable.
    /// The `page_number` variable is set to the `page_number` parameter.
    /// It makes a request to AniList with the JSON object and waits for the response.
    /// It then deserializes the response into a `SiteStatisticsMangaWrapper` and returns it along with the response string.
    ///
    /// # Arguments
    ///
    /// * `page_number` - A 64-bit integer that represents the page number.
    ///
    /// # Returns
    ///
    /// * `Result<(SiteStatisticsMangaWrapper, String), AppError>` - A Result that contains a tuple of `SiteStatisticsMangaWrapper` and a `String`, or an `AppError`.
    pub async fn new_manga(
        page_number: i64,
    ) -> Result<(SiteStatisticsMangaWrapper, String), AppError> {
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
        let api_response: SiteStatisticsMangaWrapper = serde_json::from_str(&res).map_err(|e| {
            AppError::new(
                format!("Error getting the manga with page {}. {}", page_number, e),
                ErrorType::WebRequest,
                ErrorResponseType::Message,
            )
        })?;
        Ok((api_response, res))
    }

    /// `has_next_page` is a function that checks if there is a next page.
    /// It takes no parameters.
    /// It returns a boolean that represents if there is a next page.
    ///
    /// This function gets the `has_next_page` field from the `page_info` of the `manga` in the `site_statistics` of the `data` and returns it.
    ///
    /// # Returns
    ///
    /// * `bool` - A boolean that represents if there is a next page.
    pub fn has_next_page(&self) -> bool {
        self.data.site_statistics.manga.page_info.has_next_page
    }
}
