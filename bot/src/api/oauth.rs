use crate::config::Config;
use axum::{
	extract::{Query, State},
	http::StatusCode,
	response::{IntoResponse, Redirect},
	Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info};

#[derive(Debug, Deserialize)]
pub struct OAuthCallbackQuery {
	code: Option<String>,
	error: Option<String>,
	error_description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TokenResponse {
	access_token: String,
	token_type: String,
	expires_in: u64,
	refresh_token: String,
	scope: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct UserInfo {
	id: String,
	username: String,
	discriminator: String,
	avatar: Option<String>,
	email: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Guild {
	id: String,
	name: String,
	icon: Option<String>,
	owner: bool,
	permissions: String,
}

/// Initiates the OAuth flow by redirecting to Discord
pub async fn oauth_login(State(config): State<Arc<Config>>) -> impl IntoResponse {
	let oauth_config = &config.api.oauth;
	
	let params = vec![
		("client_id", oauth_config.discord_client_id.as_str()),
		("redirect_uri", oauth_config.discord_redirect_uri.as_str()),
		("response_type", "code"),
		("scope", "identify guilds email"),
	];

	let query_string = serde_urlencoded::to_string(&params).unwrap();
	let discord_auth_url = format!("https://discord.com/api/oauth2/authorize?{}", query_string);

	info!("Redirecting to Discord OAuth: {}", discord_auth_url);
	Redirect::temporary(&discord_auth_url)
}

/// Handles the OAuth callback from Discord
pub async fn oauth_callback(
	State(config): State<Arc<Config>>,
	Query(query): Query<OAuthCallbackQuery>,
) -> impl IntoResponse {
	info!("OAuth callback received");

	// Check for errors
	if let Some(error) = query.error {
		error!("OAuth error: {} - {:?}", error, query.error_description);
		return Redirect::temporary(&format!(
			"{}/?error={}",
			config.api.oauth.frontend_url, error
		))
		.into_response();
	}

	// Get the authorization code
	let code = match query.code {
		Some(code) => code,
		None => {
			error!("No code provided in callback");
			return Redirect::temporary(&format!(
				"{}/?error=no_code",
				config.api.oauth.frontend_url
			))
			.into_response();
		},
	};

	info!("Received authorization code");

	// Exchange code for access token
	let token_response = match exchange_code_for_token(&config, &code).await {
		Ok(token) => token,
		Err(e) => {
			error!("Failed to exchange code for token: {}", e);
			return Redirect::temporary(&format!(
				"{}/?error=token_exchange_failed",
				config.api.oauth.frontend_url
			))
			.into_response();
		},
	};

	info!("Successfully exchanged code for token");

	// Get user info
	let user_info = match get_user_info(&token_response.access_token).await {
		Ok(user) => user,
		Err(e) => {
			error!("Failed to get user info: {}", e);
			return Redirect::temporary(&format!(
				"{}/?error=user_info_failed",
				config.api.oauth.frontend_url
			))
			.into_response();
		},
	};

	// Get user's guilds
	let guilds = match get_user_guilds(&token_response.access_token).await {
		Ok(guilds) => guilds,
		Err(e) => {
			error!("Failed to get user guilds: {}", e);
			return Redirect::temporary(&format!(
				"{}/?error=guilds_failed",
				config.api.oauth.frontend_url
			))
			.into_response();
		},
	};

	info!(
		"User {} logged in successfully with {} guilds",
		user_info.username,
		guilds.len()
	);

	// In a real implementation, you would:
	// 1. Store the token securely (e.g., in Redis or a database)
	// 2. Create a session token
	// 3. Return the session token to the frontend
	
	// For now, we'll redirect back to the frontend with a success flag
	// The frontend would need to be updated to handle this properly
	Redirect::temporary(&format!(
		"{}/#/profile?logged_in=true&user_id={}",
		config.api.oauth.frontend_url, user_info.id
	))
	.into_response()
}

/// Exchange authorization code for access token
async fn exchange_code_for_token(config: &Config, code: &str) -> Result<TokenResponse, String> {
	let oauth_config = &config.api.oauth;

	let params = [
		("client_id", oauth_config.discord_client_id.as_str()),
		("client_secret", oauth_config.discord_client_secret.as_str()),
		("grant_type", "authorization_code"),
		("code", code),
		("redirect_uri", oauth_config.discord_redirect_uri.as_str()),
	];

	let client = reqwest::Client::new();
	let response = client
		.post("https://discord.com/api/oauth2/token")
		.form(&params)
		.send()
		.await
		.map_err(|e| format!("Failed to send token request: {}", e))?;

	if !response.status().is_success() {
		let status = response.status();
		let body = response
			.text()
			.await
			.unwrap_or_else(|_| "Unable to read response".to_string());
		return Err(format!("Token exchange failed with status {}: {}", status, body));
	}

	response
		.json::<TokenResponse>()
		.await
		.map_err(|e| format!("Failed to parse token response: {}", e))
}

/// Get user information from Discord
async fn get_user_info(access_token: &str) -> Result<UserInfo, String> {
	let client = reqwest::Client::new();
	let response = client
		.get("https://discord.com/api/users/@me")
		.header("Authorization", format!("Bearer {}", access_token))
		.send()
		.await
		.map_err(|e| format!("Failed to get user info: {}", e))?;

	if !response.status().is_success() {
		let status = response.status();
		let body = response
			.text()
			.await
			.unwrap_or_else(|_| "Unable to read response".to_string());
		return Err(format!("Get user info failed with status {}: {}", status, body));
	}

	response
		.json::<UserInfo>()
		.await
		.map_err(|e| format!("Failed to parse user info: {}", e))
}

/// Get user's guilds from Discord
async fn get_user_guilds(access_token: &str) -> Result<Vec<Guild>, String> {
	let client = reqwest::Client::new();
	let response = client
		.get("https://discord.com/api/users/@me/guilds")
		.header("Authorization", format!("Bearer {}", access_token))
		.send()
		.await
		.map_err(|e| format!("Failed to get user guilds: {}", e))?;

	if !response.status().is_success() {
		let status = response.status();
		let body = response
			.text()
			.await
			.unwrap_or_else(|_| "Unable to read response".to_string());
		return Err(format!(
			"Get user guilds failed with status {}: {}",
			status, body
		));
	}

	response
		.json::<Vec<Guild>>()
		.await
		.map_err(|e| format!("Failed to parse guilds: {}", e))
}

/// Health check endpoint
pub async fn health_check() -> impl IntoResponse {
	Json(serde_json::json!({
		"status": "ok",
		"service": "kasuki-api"
	}))
}
