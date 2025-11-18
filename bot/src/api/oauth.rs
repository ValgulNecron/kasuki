use crate::api::server::ApiState;
use crate::config::Config;
use crate::database::user_session;
use axum::{
	extract::{Query, State},
	http::{header, StatusCode, HeaderMap},
	response::{IntoResponse, Redirect, Response},
	Json,
};
use chrono::{Duration, Utc};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info};
use uuid::Uuid;

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
pub async fn oauth_login(State(state): State<ApiState>) -> impl IntoResponse {
	let oauth_config = &state.config.api.oauth;
	
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
	State(state): State<ApiState>,
	Query(query): Query<OAuthCallbackQuery>,
) -> impl IntoResponse {
	info!("OAuth callback received");

	// Check for errors
	if let Some(error) = query.error {
		error!("OAuth error: {} - {:?}", error, query.error_description);
		return Redirect::temporary(&format!(
			"{}/?error={}",
			state.config.api.oauth.frontend_url, error
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
				state.config.api.oauth.frontend_url
			))
			.into_response();
		},
	};

	info!("Received authorization code");

	// Exchange code for access token
	let token_response = match exchange_code_for_token(&state.config, &code).await {
		Ok(token) => token,
		Err(e) => {
			error!("Failed to exchange code for token: {}", e);
			return Redirect::temporary(&format!(
				"{}/?error=token_exchange_failed",
				state.config.api.oauth.frontend_url
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
				state.config.api.oauth.frontend_url
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
				state.config.api.oauth.frontend_url
			))
			.into_response();
		},
	};

	info!(
		"User {} logged in successfully with {} guilds",
		user_info.username,
		guilds.len()
	);

	// Create session token
	let session_token = Uuid::new_v4().to_string();
	let token_expires_at = Utc::now() + Duration::seconds(token_response.expires_in as i64);
	
	// Store session in database
	let session = user_session::ActiveModel {
		session_token: Set(session_token.clone()),
		user_id: Set(user_info.id.clone()),
		discord_access_token: Set(token_response.access_token.clone()),
		discord_refresh_token: Set(token_response.refresh_token.clone()),
		token_expires_at: Set(token_expires_at.into()),
		created_at: Set(Utc::now().into()),
		last_used_at: Set(Utc::now().into()),
	};

	if let Err(e) = session.insert(&state.db).await {
		error!("Failed to store session in database: {}", e);
		return Redirect::temporary(&format!(
			"{}/?error=session_creation_failed",
			state.config.api.oauth.frontend_url
		))
		.into_response();
	}

	info!("Session created and stored for user {}", user_info.id);

	// Create response with session cookie
	let mut response = Redirect::temporary(&format!(
		"{}/#/profile",
		state.config.api.oauth.frontend_url
	)).into_response();

	// Set secure HTTP-only cookie with session token
	let cookie_value = format!(
		"session_token={}; Path=/; HttpOnly; SameSite=Lax; Max-Age={}",
		session_token,
		60 * 60 * 24 * 7 // 7 days
	);
	
	if let Ok(headers) = response.headers_mut().try_insert(
		header::SET_COOKIE,
		cookie_value.parse().unwrap()
	) {
		info!("Session cookie set successfully");
	} else {
		error!("Failed to set session cookie");
	}

	response
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

#[derive(Debug, Serialize)]
struct SessionValidationResponse {
	valid: bool,
	user: Option<SessionUser>,
}

#[derive(Debug, Serialize)]
struct SessionUser {
	id: String,
	username: String,
	avatar_url: String,
	guilds: Vec<SessionGuild>,
}

#[derive(Debug, Serialize)]
struct SessionGuild {
	id: String,
	name: String,
	icon_url: Option<String>,
}

/// Validates a session token and returns user info if valid
pub async fn validate_session(
	State(state): State<ApiState>,
	headers: HeaderMap,
) -> impl IntoResponse {
	// Extract session token from cookies
	let session_token = match extract_session_token(&headers) {
		Some(token) => token,
		None => {
			return Json(SessionValidationResponse {
				valid: false,
				user: None,
			}).into_response();
		},
	};

	// Look up session in database
	let session = match user_session::Entity::find_by_id(session_token.clone())
		.one(&state.db)
		.await
	{
		Ok(Some(session)) => session,
		Ok(None) => {
			info!("Session not found: {}", session_token);
			return Json(SessionValidationResponse {
				valid: false,
				user: None,
			}).into_response();
		},
		Err(e) => {
			error!("Database error while validating session: {}", e);
			return Json(SessionValidationResponse {
				valid: false,
				user: None,
			}).into_response();
		},
	};

	// Check if token is expired
	let now = Utc::now();
	let token_expires_at: chrono::DateTime<Utc> = session.token_expires_at.into();
	
	let access_token = if token_expires_at <= now {
		// Token expired, try to refresh
		info!("Access token expired for user {}, attempting refresh", session.user_id);
		match refresh_access_token(&state.config, &session.discord_refresh_token).await {
			Ok(new_token_response) => {
				// Update session with new tokens
				let mut session_active: user_session::ActiveModel = session.clone().into();
				session_active.discord_access_token = Set(new_token_response.access_token.clone());
				session_active.discord_refresh_token = Set(new_token_response.refresh_token.clone());
				session_active.token_expires_at = Set((Utc::now() + Duration::seconds(new_token_response.expires_in as i64)).into());
				session_active.last_used_at = Set(Utc::now().into());
				
				if let Err(e) = session_active.update(&state.db).await {
					error!("Failed to update session with refreshed token: {}", e);
					return Json(SessionValidationResponse {
						valid: false,
						user: None,
					}).into_response();
				}
				
				info!("Successfully refreshed access token for user {}", session.user_id);
				new_token_response.access_token
			},
			Err(e) => {
				error!("Failed to refresh token: {}", e);
				// Delete invalid session
				let _ = user_session::Entity::delete_by_id(session_token)
					.exec(&state.db)
					.await;
				return Json(SessionValidationResponse {
					valid: false,
					user: None,
				}).into_response();
			},
		}
	} else {
		// Update last_used_at
		let mut session_active: user_session::ActiveModel = session.clone().into();
		session_active.last_used_at = Set(Utc::now().into());
		let _ = session_active.update(&state.db).await;
		session.discord_access_token.clone()
	};

	// Get current user info
	let user_info = match get_user_info(&access_token).await {
		Ok(user) => user,
		Err(e) => {
			error!("Failed to get user info: {}", e);
			return Json(SessionValidationResponse {
				valid: false,
				user: None,
			}).into_response();
		},
	};

	// Get user's guilds
	let guilds = match get_user_guilds(&access_token).await {
		Ok(guilds) => guilds,
		Err(e) => {
			error!("Failed to get user guilds: {}", e);
			return Json(SessionValidationResponse {
				valid: false,
				user: None,
			}).into_response();
		},
	};

	// Build avatar URL
	let avatar_url = if let Some(avatar_hash) = user_info.avatar {
		format!(
			"https://cdn.discordapp.com/avatars/{}/{}.png",
			user_info.id, avatar_hash
		)
	} else {
		// Default Discord avatar
		let discriminator: u32 = user_info.discriminator.parse().unwrap_or(0);
		format!(
			"https://cdn.discordapp.com/embed/avatars/{}.png",
			discriminator % 5
		)
	};

	// Convert guilds to session guilds
	let session_guilds: Vec<SessionGuild> = guilds
		.into_iter()
		.map(|guild| {
			let icon_url = guild.icon.map(|icon_hash| {
				format!(
					"https://cdn.discordapp.com/icons/{}/{}.png",
					guild.id, icon_hash
				)
			});
			SessionGuild {
				id: guild.id,
				name: guild.name,
				icon_url,
			}
		})
		.collect();

	Json(SessionValidationResponse {
		valid: true,
		user: Some(SessionUser {
			id: user_info.id,
			username: user_info.username,
			avatar_url,
			guilds: session_guilds,
		}),
	}).into_response()
}

/// Refresh an expired access token using the refresh token
async fn refresh_access_token(config: &Config, refresh_token: &str) -> Result<TokenResponse, String> {
	let oauth_config = &config.api.oauth;

	let params = [
		("client_id", oauth_config.discord_client_id.as_str()),
		("client_secret", oauth_config.discord_client_secret.as_str()),
		("grant_type", "refresh_token"),
		("refresh_token", refresh_token),
	];

	let client = reqwest::Client::new();
	let response = client
		.post("https://discord.com/api/oauth2/token")
		.form(&params)
		.send()
		.await
		.map_err(|e| format!("Failed to send refresh token request: {}", e))?;

	if !response.status().is_success() {
		let status = response.status();
		let body = response
			.text()
			.await
			.unwrap_or_else(|_| "Unable to read response".to_string());
		return Err(format!("Token refresh failed with status {}: {}", status, body));
	}

	response
		.json::<TokenResponse>()
		.await
		.map_err(|e| format!("Failed to parse refresh token response: {}", e))
}

/// Extract session token from cookie header
fn extract_session_token(headers: &HeaderMap) -> Option<String> {
	let cookie_header = headers.get(header::COOKIE)?;
	let cookie_str = cookie_header.to_str().ok()?;
	
	// Parse cookies to find session_token
	for cookie in cookie_str.split(';') {
		let cookie = cookie.trim();
		if let Some(value) = cookie.strip_prefix("session_token=") {
			return Some(value.to_string());
		}
	}
	
	None
}

/// Logout endpoint - deletes session and clears cookie
pub async fn logout(
	State(state): State<ApiState>,
	headers: HeaderMap,
) -> impl IntoResponse {
	// Extract session token from cookies
	if let Some(session_token) = extract_session_token(&headers) {
		// Delete session from database
		let _ = user_session::Entity::delete_by_id(session_token)
			.exec(&state.db)
			.await;
	}

	// Create response with expired cookie to clear it
	let mut response = Json(serde_json::json!({
		"success": true,
		"message": "Logged out successfully"
	})).into_response();

	// Clear session cookie by setting it to expire immediately
	let cookie_value = "session_token=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0";
	
	if let Ok(_) = response.headers_mut().try_insert(
		header::SET_COOKIE,
		cookie_value.parse().unwrap()
	) {
		info!("Session cookie cleared");
	}

	response
}
