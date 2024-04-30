use serde::Deserialize;
use serde_json::json;

use crate::common::make_anilist_request::make_request_anilist;
use crate::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};

#[derive(Debug, Deserialize, Clone)]
pub struct Name {
    pub full: Option<String>,
    pub native: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Image {
    pub large: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Date {
    pub year: Option<i32>,
    pub month: Option<i32>,
    pub day: Option<i32>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Title {
    pub romaji: Option<String>,
    pub english: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Node {
    pub title: Title,
}

#[derive(Debug, Deserialize, Clone)]
pub struct StaffMedia {
    pub edges: Vec<Edge>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Edge {
    pub node: Node,
    #[serde(rename = "roleNotes")]
    pub role_notes: Option<String>,
    #[serde(rename = "relationType")]
    pub relation_type: Option<String>,
    #[serde(rename = "staffRole")]
    pub staff_role: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Character {
    pub name: Name,
    pub image: Image,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Characters {
    pub nodes: Vec<Character>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Staff {
    pub name: Name,
    pub id: i32,
    #[serde(rename = "languageV2")]
    pub language_v2: String,
    pub image: Image,
    pub description: String,
    #[serde(rename = "primaryOccupations")]
    pub primary_occupations: Vec<String>,
    pub gender: Option<String>,
    #[serde(rename = "dateOfBirth")]
    pub date_of_birth: Date,
    #[serde(rename = "dateOfDeath")]
    pub date_of_death: Date,
    pub age: Option<i32>,
    #[serde(rename = "yearsActive")]
    pub years_active: Vec<i32>,
    #[serde(rename = "homeTown")]
    pub home_town: Option<String>,
    #[serde(rename = "siteUrl")]
    pub site_url: String,
    #[serde(rename = "staffMedia")]
    pub staff_media: StaffMedia,
    pub characters: Characters,
}

#[derive(Debug, Deserialize, Clone)]
pub struct StaffData {
    #[serde(rename = "Staff")]
    pub staff: Staff,
}

#[derive(Debug, Deserialize, Clone)]
pub struct StaffWrapper {
    pub data: StaffData,
}

/// `StaffWrapper` is an implementation block for the `StaffWrapper` struct.
impl StaffWrapper {
    /// `new_staff_by_id` is an asynchronous function that creates a new staff by ID.
    /// It takes an `id` as a parameter.
    /// `id` is a 32-bit integer that represents the ID of the staff.
    /// It returns a `Result` that contains a `StaffWrapper` or an `AppError`.
    ///
    /// This function first defines a GraphQL query string that takes an `id` as a variable.
    /// It then creates a JSON object with the query string and the variable.
    /// The `id` variable is set to the `id` parameter.
    /// It makes a request to AniList with the JSON object and waits for the response.
    /// It then deserializes the response into a `StaffWrapper` and returns it.
    ///
    /// # Arguments
    ///
    /// * `id` - A 32-bit integer that represents the ID of the staff.
    ///
    /// # Returns
    ///
    /// * `Result<StaffWrapper, AppError>` - A Result that contains a `StaffWrapper` or an `AppError`.
    pub async fn new_staff_by_id(id: i32) -> Result<StaffWrapper, AppError> {
        let query_id: &str = "
query ($name: Int, $limit1: Int = 5, $limit2: Int = 15) {
	Staff(id: $name){
    name {
      full
      native
    }
    id
    languageV2
    image {
      large
    }
    description
    primaryOccupations
    gender
    dateOfBirth {
      year
      month
      day
    }
    dateOfDeath {
      year
      month
      day
    }
    age
    yearsActive
    homeTown
    siteUrl
    staffMedia(perPage: $limit1){
      edges{
        node {
          title {
            romaji
            english
          }
        }
        roleNotes
        relationType
        staffRole
      }
    }
    characters(perPage: $limit2) {
      nodes {
        name {
          full
        }
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
    /// It returns a `Result` that contains a `StaffWrapper` or an `AppError`.
    ///
    /// This function first defines a GraphQL query string that takes a `search` as a variable.
    /// It then creates a JSON object with the query string and the variable.
    /// The `search` variable is set to the `search` parameter.
    /// It makes a request to AniList with the JSON object and waits for the response.
    /// It then deserializes the response into a `StaffWrapper` and returns it.
    ///
    /// # Arguments
    ///
    /// * `search` - A reference to a String that represents the search query.
    ///
    /// # Returns
    ///
    /// * `Result<StaffWrapper, AppError>` - A Result that contains a `StaffWrapper` or an `AppError`.
    pub async fn new_staff_by_search(search: &String) -> Result<StaffWrapper, AppError> {
        let query_string: &str = "
query ($name: String, $limit1: Int = 5, $limit2: Int = 15) {
	Staff(search: $name){
    name {
      full
      native
    }
    id
    languageV2
    image {
      large
    }
    description
    primaryOccupations
    gender
    dateOfBirth {
      year
      month
      day
    }
    dateOfDeath {
      year
      month
      day
    }
    age
    yearsActive
    homeTown
    siteUrl
    staffMedia(perPage: $limit1){
      edges{
        node {
          title {
            romaji
            english
          }
        }
        roleNotes
        relationType
        staffRole
      }
    }
    characters(perPage: $limit2) {
      nodes {
        name {
          full
        }
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
