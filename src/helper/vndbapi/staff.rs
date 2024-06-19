use serde::{Deserialize, Serialize};

pub async fn get_staff(
    value: String,
) -> Result<StaffRoot, crate::helper::error_management::error_enum::AppError> {
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
    let response =
        crate::helper::vndbapi::common::do_request_cached_with_json(path.clone(), json.to_string())
            .await?;
    let response: StaffRoot = serde_json::from_str(&response).map_err(|e| {
        crate::helper::error_management::error_enum::AppError {
            message: format!("Error while parsing response: '{}'", e),
            error_type: crate::helper::error_management::error_enum::ErrorType::WebRequest,
            error_response_type:
                crate::helper::error_management::error_enum::ErrorResponseType::Unknown,
        }
    })?;
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
