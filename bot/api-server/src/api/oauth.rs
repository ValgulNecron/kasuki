use axum::{
	extract::{Query, State},
	response::{IntoResponse, Redirect},
	Json,
};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use chrono::{Duration, Utc};
// Added
use dashmap::DashMap;
use jsonwebtoken::{encode, EncodingKey, Header};
// Added
use serde::{Deserialize, Serialize};
use shared::config::Config;
use std::sync::Arc;
use std::sync::LazyLock;
use tracing::{debug, error, info, trace, warn};

// Global cache for user data, mapping user ID to (UserInfo, Vec<Guild>)
static USER_CACHE: LazyLock<DashMap<String, (UserInfo, Vec<Guild>)>> = LazyLock::new(DashMap::new);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
	pub sub: String,      // Subject (user ID)
	pub username: String, // Username
	pub exp: usize,       // Expiration time
}

#[derive(Debug, Deserialize)]
pub struct OAuthCallbackQuery {
	code: Option<String>,
	error: Option<String>,
	error_description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TokenResponse {
	access_token: String,
	token_type: String,
	expires_in: u64,
	refresh_token: String,
	scope: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserInfo {
	pub id: String,
	pub username: String,
	pub discriminator: String,
	pub avatar: Option<String>,
	pub email: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Guild {
	pub id: String,
	pub name: String,
	pub icon_hash: Option<String>, // Renamed from 'icon'
	pub icon_url: Option<String>,  // New field for the full URL
	pub owner: bool,
	pub permissions: String,
}

// Struct to deserialize the raw Discord API response for guilds
#[derive(Debug, Deserialize)]
struct RawDiscordGuild {
	id: String,
	name: String,
	icon: Option<String>, // Raw icon hash from Discord
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

	debug!("Redirecting to Discord OAuth: {}", discord_auth_url);
	Redirect::temporary(&discord_auth_url)
}

/// Handles the OAuth callback from Discord
pub async fn oauth_callback(
	State(config): State<Arc<Config>>, Query(query): Query<OAuthCallbackQuery>,
) -> impl IntoResponse {
	trace!("OAuth callback received");

	// Check for errors
	if let Some(error) = query.error {
		warn!("OAuth error: {} - {:?}", error, query.error_description);
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
			warn!("No code provided in callback");
			return Redirect::temporary(&format!(
				"{}/?error=no_code",
				config.api.oauth.frontend_url
			))
			.into_response();
		},
	};

	trace!("Received authorization code");

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

	trace!("Successfully exchanged code for token");

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

	// Store user info and guilds in the cache
	USER_CACHE.insert(user_info.id.clone(), (user_info.clone(), guilds.clone()));
	debug!("User data for {} cached.", user_info.id);

	// Generate JWT
	let jwt_secret = &config.api.oauth.jwt_secret;
	let expiration = Utc::now() + Duration::hours(24); // Token valid for 24 hours
	let claims = Claims {
		sub: user_info.id.clone(),
		username: user_info.username.clone(),
		exp: expiration.timestamp() as usize,
	};

	let secret = STANDARD.decode(jwt_secret).unwrap();

	let token = match encode(
		&Header::default(),
		&claims,
		&EncodingKey::from_secret(&secret),
	) {
		Ok(t) => t,
		Err(e) => {
			error!("Failed to generate JWT: {}", e);
			return Redirect::temporary(&format!(
				"{}/?error=jwt_generation_failed",
				config.api.oauth.frontend_url
			))
			.into_response();
		},
	};
	debug!("Generated JWT for user {}", user_info.username);

	// Redirect back to the frontend with the JWT
	Redirect::temporary(&format!(
		"{}/#/profile?jwt={}",
		config.api.oauth.frontend_url, token
	))
	.into_response()
}

/// Retrieve cached user data
pub fn get_cached_user_data(user_id: &str) -> Option<(UserInfo, Vec<Guild>)> {
	USER_CACHE.get(user_id).map(|entry| entry.to_owned())
}

/// Exchange authorization code for access token
async fn exchange_code_for_token(
	config: &Config, code: &str,
) -> Result<TokenResponse, Box<dyn std::error::Error>> {
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
		.post("https://discord.com/api/v10/oauth2/token")
		.form(&params)
		.send()
		.await?;

	if !response.status().is_success() {
		let status = response.status();
		let body = response.text().await?;
		let err_msg = format!("Token exchange failed with status {}: {}", status, body);
		error!("{}", err_msg);
		return Err(err_msg.into());
	}

	response.json::<TokenResponse>().await.map_err(|e| {
		error!("Failed to parse token response: {}", e);
		e.into()
	})
}

/// Get user information from Discord
async fn get_user_info(access_token: &str) -> Result<UserInfo, Box<dyn std::error::Error>> {
	let client = reqwest::Client::new();
	let response = client
		.get("https://discord.com/api/v10/users/@me")
		.header("Authorization", format!("Bearer {}", access_token))
		.send()
		.await?;

	if !response.status().is_success() {
		let status = response.status();
		let body = response.text().await?;
		let err_msg = format!("Get user info failed with status {}: {}", status, body);
		error!("{}", err_msg);
		return Err(err_msg.into());
	}

	response.json::<UserInfo>().await.map_err(|e| {
		error!("Failed to parse user info: {}", e);
		e.into()
	})
}

/// Get user's guilds from Discord
async fn get_user_guilds(access_token: &str) -> Result<Vec<Guild>, Box<dyn std::error::Error>> {
	let client = reqwest::Client::new();
	let response = client
		.get("https://discord.com/api/v10/users/@me/guilds")
		.header("Authorization", format!("Bearer {}", access_token))
		.send()
		.await?;

	if !response.status().is_success() {
		let status = response.status();
		let body = response.text().await?;
		let err_msg = format!("Get user guilds failed with status {}: {}", status, body);
		error!("{}", err_msg);
		return Err(err_msg.into());
	}

	response
		.json::<Vec<RawDiscordGuild>>()
		.await
		.map_err(|e| {
			error!("Failed to parse guilds: {}", e);
			e.into()
		})
		.map(|raw_guilds| {
			raw_guilds
				.into_iter()
				.map(|raw_guild| {
					let icon_url = if let Some(icon_hash) = raw_guild.icon.clone() {
						Some(format!(
							"https://cdn.discordapp.com/icons/{}/{}.png",
							raw_guild.id, icon_hash
						))
					} else {
						None
					};
					Guild {
						id: raw_guild.id,
						name: raw_guild.name,
						icon_hash: raw_guild.icon,
						icon_url,
						owner: raw_guild.owner,
						permissions: raw_guild.permissions,
					}
				})
				.collect()
		})
}

/// Health check endpoint
pub async fn health_check() -> impl IntoResponse {
	Json(serde_json::json!({
		"status": "ok",
		"service": "kasuki-api"
	}))
}
