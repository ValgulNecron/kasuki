use serde::Deserialize;
use serde_json::json;

use crate::common::make_anilist_request::make_request_anilist;
use crate::error_management::api_request_error::ApiRequestError;
use crate::error_management::api_request_error::ApiRequestError::NotFound;

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

impl StaffImageWrapper {
    pub async fn new_staff_by_id(id: i32) -> Result<StaffImageWrapper, ApiRequestError> {
        let query_id: &str = "
        query ($name: Int, $limit: Int = 4) {
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
        serde_json::from_str(&resp)
            .map_err(|e| NotFound(format!("Error getting the staff with id {}. {}", id, e)))
    }

    pub async fn new_staff_by_search(
        search: &String,
    ) -> Result<StaffImageWrapper, ApiRequestError> {
        let query_string: &str = "
query ($name: String, $limit: Int = 4) {
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
            NotFound(format!(
                "Error getting the staff with name {}. {}",
                search, e
            ))
        })
    }
}
