use leptos::logging::log;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Request, RequestInit, RequestMode, Response};
use serde::{Deserialize, Serialize};
use crate::config::Config;
use crate::app::{User, Guild}; // Import User and Guild from app.rs

// This struct will represent the expected response from our backend API
// This should match the UserDataResponse in bot/src/api/server.rs
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UserDataResponse {
    pub user: User,
    pub guilds: Vec<Guild>,
}

pub async fn fetch_user_data(jwt: Option<String>) -> Result<(User, Vec<Guild>), JsValue> { // Removed user_id
    let api_base_url = Config::api_url();
    let url = format!("{}/api/user/me", api_base_url);

    log!("Fetching user data from: {}", url);

    let opts = RequestInit::new();
    opts.set_method("GET");
    opts.set_mode(RequestMode::Cors);

    let headers = web_sys::Headers::new()?;
    if let Some(token) = jwt {
        headers.set("Authorization", &format!("Bearer {}", token))?;
    } else {
        log!("No JWT provided for fetch_user_data");
    }
    opts.set_headers(&headers); // Set headers on RequestInit

    let request = Request::new_with_str_and_init(&url, &opts)?;

    let window = web_sys::window().ok_or_else(|| JsValue::from_str("no global `window` exists"))?;
    let resp_value = wasm_bindgen_futures::JsFuture::from(window.fetch_with_request(&request)).await?;

    // `resp_value` is a `Response` object.
    let resp: Response = resp_value.dyn_into().unwrap();

    // Check if the response was successful
    if !resp.ok() {
        let error_text = wasm_bindgen_futures::JsFuture::from(resp.text()?).await?;
        return Err(JsValue::from_str(&format!("Failed to fetch user data: {}", error_text.as_string().unwrap_or_default())));
    }

    let json = wasm_bindgen_futures::JsFuture::from(resp.json()?).await?;
    let user_data_response: UserDataResponse = serde_wasm_bindgen::from_value(json)?;

    Ok((user_data_response.user, user_data_response.guilds))
}