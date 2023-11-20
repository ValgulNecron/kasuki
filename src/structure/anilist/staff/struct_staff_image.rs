use crate::function::requests::request::make_request_anilist;
use log::error;
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Deserialize)]
pub struct StaffImageWrapper {
    data: StaffImageData,
}

#[derive(Debug, Deserialize)]
pub struct StaffImageData {
    #[serde(rename = "Staff")]
    staff: Staff,
}

#[derive(Debug, Deserialize)]
pub struct Staff {
    id: i32,
    images: StaffImageImage,
    characters: StaffImageCharacters,
}

#[derive(Debug, Deserialize)]
pub struct StaffImageImage {
    large: String,
}

#[derive(Debug, Deserialize)]
pub struct StaffImageCharacters {
    nodes: Vec<StaffImageNodes>,
}

#[derive(Debug, Deserialize)]
pub struct StaffImageNodes {
    image: StaffImageImage,
}

impl StaffImageWrapper {
    pub async fn new_staff_by_id(id: i32) -> Result<StaffImageWrapper, String> {
        let query_id: &str = "
        query ($name: Int, $limit: Int = 4) {
	Staff(id: $name){
    id
    image{
      large
    }
    characters(perPage: $limit) {
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
        match serde_json::from_str(&resp) {
            Ok(result) => Ok(result),
            Err(e) => {
                error!("Failed to parse JSON: {}", e);
                Err(String::from("Error: Failed to retrieve user data"))
            }
        }
    }

    pub async fn new_staff_by_search(search: &String) -> Result<StaffImageWrapper, String> {
        let query_string: &str = "
query ($name: String, $limit: Int = 4) {
	Staff(search: $name){
    id
    image{
      large
    }
    characters(perPage: $limit) {
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
        match serde_json::from_str(&resp) {
            Ok(result) => Ok(result),
            Err(e) => {
                error!("Failed to parse JSON: {}", e);
                Err(String::from("Error: Failed to retrieve user data"))
            }
        }
    }

    pub fn get_staff_image(&self) -> &String {
        &self.data.staff.images.large
    }

    pub fn get_characters_image(&self) -> &Vec<StaffImageNodes> {
        &self.data.staff.characters.nodes
    }
}
