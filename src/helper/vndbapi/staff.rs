use std::error::Error;
use std::sync::Arc;

use moka::future::Cache;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::helper::error_management::error_enum::UnknownResponseError;

pub async fn get_staff(
    value: String,
    vndb_cache: Arc<RwLock<Cache<String, String>>>,
) -> Result<StaffRoot, Box<dyn Error>> {
    let value = value.to_lowercase();
    let value = value.trim();
    let start_with_v = value.starts_with('v');
    let is_number = value.chars().skip(1).all(|c| c.is_numeric());
    let json = if start_with_v && is_number {
        (r#"{
    		"filters": ["id", "=",""#
            .to_owned()
            + value
            + r#""],
    		"fields": "id,aid,ismain,name,lang,gender,description"
		}"#)
            .to_string()
    } else {
        (r#"{
    		"filters": ["search", "=",""#
            .to_owned()
            + value
            + r#""],
    		"fields": "id,aid,ismain,name,lang,gender,description"
		}"#)
            .to_string()
    };
    let path = "/staff".to_string();
    let response = crate::helper::vndbapi::common::do_request_cached_with_json(
        path.clone(),
        json.to_string(),
        vndb_cache,
    )
        .await?;
    let response: StaffRoot = serde_json::from_str(&response)
        .map_err(|e| UnknownResponseError::Json(format!("{:#?}", e)))?;
    Ok(response)
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Staff {
    pub ismain: bool,

    pub aid: i32,

    pub name: String,

    pub gender: String,

    pub lang: String,

    pub description: String,

    pub id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StaffRoot {
    pub results: Vec<Staff>,

    pub more: bool,
}
