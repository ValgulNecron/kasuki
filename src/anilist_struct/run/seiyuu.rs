use serde::Deserialize;
use serde_json::json;

use crate::common::make_anilist_request::make_request_anilist;
use crate::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};

#[derive(Debug, Deserialize, Clone)]
pub struct StaffImageWrapper {
    pub data: StaffImageData,
}

#[derive(Debug, Deserialize, Clone)]
pub struct StaffImageData {
    #[serde(rename = "Staff")]
    pub staff: Staff,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Staff {
    pub image: StaffImageImage,
    pub characters: StaffImageCharacters,
}

#[derive(Debug, Deserialize, Clone)]
pub struct StaffImageImage {
    pub large: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct StaffImageCharacters {
    pub nodes: Vec<StaffImageNodes>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct StaffImageNodes {
    pub image: StaffImageImage,
}

/// `StaffImageWrapper` is an implementation block for the `StaffImageWrapper` struct.
impl StaffImageWrapper {
    /// `new_staff_by_id` is an asynchronous function that creates a new staff by ID.
    /// It takes an `id` as a parameter.
    /// `id` is a 32-bit integer that represents the ID of the staff.
    /// It returns a `Result` that contains a `StaffImageWrapper` or an `AppError`.
    ///
    /// This function first defines a GraphQL query string that takes an `id` as a variable.
    /// It then creates a JSON object with the query string and the variable.
    /// The `id` variable is set to the `id` parameter.
    /// It makes a request to AniList with the JSON object and waits for the response.
    /// It then deserializes the response into a `StaffImageWrapper` and returns it.
    ///
    /// # Arguments
    ///
    /// * `id` - A 32-bit integer that represents the ID of the staff.
    ///
    /// # Returns
    ///
    /// * `Result<StaffImageWrapper, AppError>` - A Result that contains a `StaffImageWrapper` or an `AppError`.
    pub async fn new_staff_by_id(id: i32) -> Result<StaffImageWrapper, AppError> {
        let query_id: &str = "
        query ($name: Int, $limit: Int = 9) {
	Staff(id: $name){
    image{
      large
    }
    characters(perPage: $limit, sort: FAVOURITES_DESC) {
      nodes {
        image {
          large
        }
      }
    }
  }
}
";
        let json = json!({"query": query_id, "variables": {"name": id}});
        let resp = make_request_anilist(json, false).await;
        serde_json::from_str(&resp).map_err(|e| {
            AppError::new(
                format!("Error getting the staff with id {}. {}", id, e),
                ErrorType::WebRequest,
                ErrorResponseType::Message,
            )
        })
    }

    /// `new_staff_by_search` is an asynchronous function that creates a new staff by search.
    /// It takes a `search` as a parameter.
    /// `search` is a reference to a String that represents the search query.
    /// It returns a `Result` that contains a `StaffImageWrapper` or an `AppError`.
    ///
    /// This function first defines a GraphQL query string that takes a `search` as a variable.
    /// It then creates a JSON object with the query string and the variable.
    /// The `search` variable is set to the `search` parameter.
    /// It makes a request to AniList with the JSON object and waits for the response.
    /// It then deserializes the response into a `StaffImageWrapper` and returns it.
    ///
    /// # Arguments
    ///
    /// * `search` - A reference to a String that represents the search query.
    ///
    /// # Returns
    ///
    /// * `Result<StaffImageWrapper, AppError>` - A Result that contains a `StaffImageWrapper` or an `AppError`.
    pub async fn new_staff_by_search(search: &String) -> Result<StaffImageWrapper, AppError> {
        let query_string: &str = "
query ($name: String, $limit: Int = 9) {
	Staff(search: $name){
    image{
      large
    }
    characters(perPage: $limit, sort: FAVOURITES_DESC) {
      nodes {
        image {
          large
        }
      }
    }
  }
}
";
        let json = json!({"query": query_string, "variables": {"name": search}});
        let resp = make_request_anilist(json, false).await;
        serde_json::from_str(&resp).map_err(|e| {
            AppError::new(
                format!("Error getting the staff with name {}. {}", search, e),
                ErrorType::WebRequest,
                ErrorResponseType::Message,
            )
        })
    }
}
