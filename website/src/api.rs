use crate::app::{Guild, User};
use crate::components::activities::{ActivityInfo, AnilistSearchResult};
use crate::components::server_settings::GuildSettings;
use crate::config::Config;
use leptos::logging::log;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Request, RequestInit, RequestMode, Response};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UserDataResponse {
    pub user: User,
    pub guilds: Vec<Guild>,
}

fn get_jwt_headers(jwt: Option<String>) -> Result<web_sys::Headers, JsValue> {
    let headers = web_sys::Headers::new()?;
    if let Some(token) = jwt {
        headers.set("Authorization", &format!("Bearer {}", token))?;
    }
    Ok(headers)
}

async fn fetch_json<T: for<'de> Deserialize<'de>>(
    method: &str, url: &str, jwt: Option<String>, body: Option<&str>,
) -> Result<T, JsValue> {
    let opts = RequestInit::new();
    opts.set_method(method);
    opts.set_mode(RequestMode::Cors);

    let headers = get_jwt_headers(jwt)?;
    if body.is_some() {
        headers.set("Content-Type", "application/json")?;
    }
    opts.set_headers(&headers);

    if let Some(b) = body {
        opts.set_body(&JsValue::from_str(b));
    }

    let request = Request::new_with_str_and_init(url, &opts)?;
    let window =
        web_sys::window().ok_or_else(|| JsValue::from_str("no global `window` exists"))?;
    let resp_value =
        wasm_bindgen_futures::JsFuture::from(window.fetch_with_request(&request)).await?;
    let resp: Response = resp_value.dyn_into().unwrap();

    if !resp.ok() {
        let error_text = wasm_bindgen_futures::JsFuture::from(resp.text()?).await?;
        return Err(JsValue::from_str(&format!(
            "Request failed: {}",
            error_text.as_string().unwrap_or_default()
        )));
    }

    let json = wasm_bindgen_futures::JsFuture::from(resp.json()?).await?;
    serde_wasm_bindgen::from_value(json).map_err(|e| JsValue::from_str(&e.to_string()))
}

async fn fetch_json_no_body(
    method: &str, url: &str, jwt: Option<String>,
) -> Result<(), JsValue> {
    let opts = RequestInit::new();
    opts.set_method(method);
    opts.set_mode(RequestMode::Cors);
    let headers = get_jwt_headers(jwt)?;
    opts.set_headers(&headers);

    let request = Request::new_with_str_and_init(url, &opts)?;
    let window =
        web_sys::window().ok_or_else(|| JsValue::from_str("no global `window` exists"))?;
    let resp_value =
        wasm_bindgen_futures::JsFuture::from(window.fetch_with_request(&request)).await?;
    let resp: Response = resp_value.dyn_into().unwrap();

    if !resp.ok() {
        let error_text = wasm_bindgen_futures::JsFuture::from(resp.text()?).await?;
        return Err(JsValue::from_str(&format!(
            "Request failed: {}",
            error_text.as_string().unwrap_or_default()
        )));
    }
    Ok(())
}

pub async fn exchange_auth_code(code: &str) -> Result<String, JsValue> {
    let url = format!("{}/api/oauth/token", Config::api_url());
    let body = serde_json::json!({"code": code}).to_string();

    let opts = RequestInit::new();
    opts.set_method("POST");
    opts.set_mode(RequestMode::Cors);

    let headers = web_sys::Headers::new()?;
    headers.set("Content-Type", "application/json")?;
    opts.set_headers(&headers);
    opts.set_body(&JsValue::from_str(&body));

    let request = Request::new_with_str_and_init(&url, &opts)?;
    let window =
        web_sys::window().ok_or_else(|| JsValue::from_str("no global `window` exists"))?;
    let resp_value =
        wasm_bindgen_futures::JsFuture::from(window.fetch_with_request(&request)).await?;
    let resp: Response = resp_value.dyn_into().unwrap();

    if !resp.ok() {
        let error_text = wasm_bindgen_futures::JsFuture::from(resp.text()?).await?;
        return Err(JsValue::from_str(&format!(
            "Token exchange failed: {}",
            error_text.as_string().unwrap_or_default()
        )));
    }

    let json = wasm_bindgen_futures::JsFuture::from(resp.json()?).await?;

    #[derive(serde::Deserialize)]
    struct TokenResponse {
        token: String,
    }

    let token_resp: TokenResponse =
        serde_wasm_bindgen::from_value(json).map_err(|e| JsValue::from_str(&e.to_string()))?;
    Ok(token_resp.token)
}

pub async fn fetch_user_data(jwt: Option<String>) -> Result<(User, Vec<Guild>), JsValue> {
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
    opts.set_headers(&headers);

    let request = Request::new_with_str_and_init(&url, &opts)?;

    let window =
        web_sys::window().ok_or_else(|| JsValue::from_str("no global `window` exists"))?;
    let resp_value =
        wasm_bindgen_futures::JsFuture::from(window.fetch_with_request(&request)).await?;

    let resp: Response = resp_value.dyn_into().unwrap();

    if !resp.ok() {
        let error_text = wasm_bindgen_futures::JsFuture::from(resp.text()?).await?;
        return Err(JsValue::from_str(&format!(
            "Failed to fetch user data: {}",
            error_text.as_string().unwrap_or_default()
        )));
    }

    let json = wasm_bindgen_futures::JsFuture::from(resp.json()?).await?;
    let user_data_response: UserDataResponse = serde_wasm_bindgen::from_value(json)?;

    Ok((user_data_response.user, user_data_response.guilds))
}

// ─── Guild API ────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct AdminGuild {
    pub id: String,
    pub name: String,
    pub icon_url: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct LangOption {
    pub code: String,
    pub name: String,
}

pub async fn fetch_admin_guilds(jwt: Option<String>) -> Result<Vec<AdminGuild>, JsValue> {
    let url = format!("{}/api/guild/admin", Config::api_url());
    fetch_json("GET", &url, jwt, None).await
}

pub async fn fetch_langs(jwt: Option<String>) -> Result<Vec<LangOption>, JsValue> {
    let url = format!("{}/api/guild/langs", Config::api_url());
    fetch_json("GET", &url, jwt, None).await
}

pub async fn fetch_guild_settings(
    jwt: Option<String>, guild_id: &str,
) -> Result<GuildSettings, JsValue> {
    let url = format!("{}/api/guild/{}/settings", Config::api_url(), guild_id);
    fetch_json("GET", &url, jwt, None).await
}

pub async fn update_guild_lang(
    jwt: Option<String>, guild_id: &str, lang: &str,
) -> Result<(), JsValue> {
    let url = format!("{}/api/guild/{}/lang", Config::api_url(), guild_id);
    let body = serde_json::json!({"lang": lang}).to_string();
    let _: serde_json::Value = fetch_json("PUT", &url, jwt, Some(&body)).await?;
    Ok(())
}

pub async fn update_guild_modules(
    jwt: Option<String>, guild_id: &str, body: String,
) -> Result<(), JsValue> {
    let url = format!("{}/api/guild/{}/modules", Config::api_url(), guild_id);
    let _: serde_json::Value = fetch_json("PUT", &url, jwt, Some(&body)).await?;
    Ok(())
}

// ─── Activity API ─────────────────────────────────────────

pub async fn fetch_guild_activities(
    jwt: Option<String>, guild_id: &str,
) -> Result<Vec<ActivityInfo>, JsValue> {
    let url = format!(
        "{}/api/guild/{}/activities",
        Config::api_url(),
        guild_id
    );
    fetch_json("GET", &url, jwt, None).await
}

pub async fn add_activity(
    jwt: Option<String>, guild_id: &str, body: String,
) -> Result<(), JsValue> {
    let url = format!(
        "{}/api/guild/{}/activities",
        Config::api_url(),
        guild_id
    );
    let _: serde_json::Value = fetch_json("POST", &url, jwt, Some(&body)).await?;
    Ok(())
}

pub async fn delete_activity(
    jwt: Option<String>, guild_id: &str, anime_id: i32,
) -> Result<(), JsValue> {
    let url = format!(
        "{}/api/guild/{}/activities/{}",
        Config::api_url(),
        guild_id,
        anime_id
    );
    fetch_json_no_body("DELETE", &url, jwt).await
}

pub async fn search_anilist(
    jwt: Option<String>, query: &str,
) -> Result<Vec<AnilistSearchResult>, JsValue> {
    let encoded = js_sys::encode_uri_component(query);
    let url = format!(
        "{}/api/anilist/search?q={}",
        Config::api_url(),
        encoded
    );
    fetch_json("GET", &url, jwt, None).await
}

// ─── Blacklist API ────────────────────────────────────────

pub async fn request_blacklist(jwt: Option<String>) -> Result<(), JsValue> {
    let url = format!("{}/api/user/blacklist-request", Config::api_url());
    let _: serde_json::Value = fetch_json("POST", &url, jwt, None).await?;
    Ok(())
}
