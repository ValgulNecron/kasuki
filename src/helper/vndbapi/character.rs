use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Image {
    pub sexual: f64,
    pub url: String,
    pub violence: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VN {
    pub id: String,
    pub title: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Trait {
    pub spoiler: i64,
    pub name: String,
    pub id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Character {
    pub blood_type: Option<String>,
    pub description: Option<String>,
    pub traits: Vec<Trait>,
    pub waist: Option<i64>,
    pub name: String,
    pub height: Option<i64>,
    pub cup: Option<String>,
    pub sex: Vec<String>,
    pub vns: Vec<VN>,
    pub image: Option<Image>,
    pub hips: Option<i64>,
    pub id: String,
    pub bust: Option<i64>,
    pub weight: Option<i64>,
    pub age: Option<i64>,
    pub birthday: Option<Vec<i32>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CharacterRoot {
    pub more: bool,
    pub results: Vec<Character>,
}
use moka::future::Cache;
use std::sync::Arc;
use tokio::sync::RwLock;

pub async fn get_character(
    value: String,
    vndb_cache: Arc<RwLock<Cache<String, String>>>,
) -> Result<CharacterRoot, crate::helper::error_management::error_enum::AppError> {
    let value = value.to_lowercase();
    let value = value.trim();
    let start_with_v = value.starts_with('v');
    let is_number = value.chars().skip(1).all(|c| c.is_numeric());
    let json = if start_with_v && is_number {
        (r#"{
    		"filters": ["id", "=",""#.to_owned() + value + r#""],
    		"fields": "id,description, name,image.url,image.sexual,image.violence,blood_type,height,weight,bust,waist,hips,cup,age,sex,vns.title,traits.spoiler,traits.name"
		}"#).to_string()
    } else {
        (r#"{
    		"filters": ["search", "=",""#.to_owned() + value + r#""],
    		"fields": "id,description, name,image.url,image.sexual,image.violence,blood_type,height,weight,bust,waist,hips,cup,age,sex,vns.title,traits.spoiler,traits.name"
		}"#).to_string()
    };
    let path = "/character".to_string();
    let response = crate::helper::vndbapi::common::do_request_cached_with_json(
        path.clone(),
        json.to_string(),
        vndb_cache,
    )
    .await?;
    let response: CharacterRoot = serde_json::from_str(&response).map_err(|e| {
        crate::helper::error_management::error_enum::AppError {
            message: format!("Error while parsing response: '{}'", e),
            error_type: crate::helper::error_management::error_enum::ErrorType::WebRequest,
            error_response_type:
                crate::helper::error_management::error_enum::ErrorResponseType::Unknown,
        }
    })?;
    Ok(response)
}
