use axum::{
	extract::{Query, State},
	response::{IntoResponse, Redirect},
	Json,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, Header};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, EntityTrait};
use serde::{Deserialize, Serialize};
use shared::database::oauth_token;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::api::error::AppError;
use crate::api::state::{AppState, AuthCodeEntry};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
	pub sub: String,
	pub username: String,
	pub exp: usize,
}

#[derive(Debug, Deserialize)]
pub struct OAuthCallbackQuery {
	code: Option<String>,
	state: Option<String>,
	error: Option<String>,
	error_description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TokenExchangeRequest {
	pub code: String,
}

#[derive(Debug, Serialize)]
pub struct TokenExchangeResponse {
	pub token: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TokenResponse {
	pub access_token: String,
	pub token_type: String,
	pub expires_in: u64,
	pub refresh_token: String,
	pub scope: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserInfo {
	pub id: String,
	pub username: String,
	#[serde(skip_serializing)]
	#[allow(dead_code)]
	pub discriminator: String,
	pub avatar: Option<String>,
	#[serde(skip_serializing)]
	#[allow(dead_code)]
	pub email: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Guild {
	pub id: String,
	pub name: String,
	#[serde(skip_serializing)]
	#[allow(dead_code)]
	pub icon_hash: Option<String>,
	pub icon_url: Option<String>,
	#[serde(skip_serializing)]
	#[allow(dead_code)]
	pub owner: bool,
	#[serde(skip_serializing)]
	#[allow(dead_code)]
	pub permissions: String,
}

#[derive(Debug, Deserialize)]
struct RawDiscordGuild {
	id: String,
	name: String,
	icon: Option<String>,
	owner: bool,
	permissions: String,
}

pub async fn oauth_login(State(state): State<AppState>) -> impl IntoResponse {
	let oauth_config = &state.config.api.oauth;

	let csrf_state = Uuid::new_v4().to_string();
	state.oauth_states.insert(csrf_state.clone(), ()).await;

	let params = vec![
		("client_id", oauth_config.discord_client_id.as_str()),
		("redirect_uri", oauth_config.discord_redirect_uri.as_str()),
		("response_type", "code"),
		("scope", "identify guilds email"),
		("state", csrf_state.as_str()),
	];

	let query_string = serde_urlencoded::to_string(&params)
		.expect("failed to encode OAuth query params");
	let discord_auth_url = format!("https://discord.com/api/oauth2/authorize?{}", query_string);

	debug!("redirecting to discord oauth");
	Redirect::temporary(&discord_auth_url)
}

pub async fn oauth_callback(
	State(state): State<AppState>, Query(query): Query<OAuthCallbackQuery>,
) -> impl IntoResponse {
	if let Some(error) = query.error {
		warn!(error = %error, description = ?query.error_description, "discord oauth error");
		return Redirect::temporary(&format!(
			"{}/?error={}",
			state.config.api.oauth.frontend_url, error
		))
		.into_response();
	}

	let csrf_state = match query.state {
		Some(s) => s,
		None => {
			warn!("oauth callback missing state parameter");
			return Redirect::temporary(&format!(
				"{}/?error=missing_state",
				state.config.api.oauth.frontend_url
			))
			.into_response();
		},
	};

	if state.oauth_states.remove(&csrf_state).await.is_none() {
		warn!(state = %csrf_state, "invalid or expired csrf state");
		return Redirect::temporary(&format!(
			"{}/?error=invalid_state",
			state.config.api.oauth.frontend_url
		))
		.into_response();
	}

	let code = match query.code {
		Some(code) => code,
		None => {
			warn!("oauth callback missing authorization code");
			return Redirect::temporary(&format!(
				"{}/?error=no_code",
				state.config.api.oauth.frontend_url
			))
			.into_response();
		},
	};

	let token_response = match exchange_code_for_token(&state, &code).await {
		Ok(token) => token,
		Err(e) => {
			error!(error = %e.message, "discord token exchange failed");
			return Redirect::temporary(&format!(
				"{}/?error=token_exchange_failed",
				state.config.api.oauth.frontend_url
			))
			.into_response();
		},
	};

	let user_info = match get_user_info(&state.http_client, &token_response.access_token).await {
		Ok(user) => user,
		Err(e) => {
			error!(error = %e.message, "discord user info request failed");
			return Redirect::temporary(&format!(
				"{}/?error=user_info_failed",
				state.config.api.oauth.frontend_url
			))
			.into_response();
		},
	};

	let guilds = match get_user_guilds(&state.http_client, &token_response.access_token).await {
		Ok(guilds) => guilds,
		Err(e) => {
			error!(error = %e.message, "discord guilds request failed");
			return Redirect::temporary(&format!(
				"{}/?error=guilds_failed",
				state.config.api.oauth.frontend_url
			))
			.into_response();
		},
	};

	info!(
		user = %user_info.username,
		guilds = guilds.len(),
		"user authenticated via discord"
	);

	if let Err(e) = store_oauth_tokens(&state, &user_info.id, &token_response).await {
		error!(error = %e.message, user = %user_info.id, "failed to persist oauth tokens");
	}

	state
		.user_cache
		.insert(user_info.id.clone(), (user_info.clone(), guilds))
		.await;

	let auth_code = Uuid::new_v4().to_string();
	state
		.auth_codes
		.insert(
			auth_code.clone(),
			AuthCodeEntry {
				user_id: user_info.id.clone(),
			},
		)
		.await;

	Redirect::temporary(&format!(
		"{}/#/profile?code={}",
		state.config.api.oauth.frontend_url, auth_code
	))
	.into_response()
}

pub async fn exchange_auth_code(
	State(state): State<AppState>, Json(body): Json<TokenExchangeRequest>,
) -> Result<Json<TokenExchangeResponse>, AppError> {
	let entry = state
		.auth_codes
		.remove(&body.code)
		.await
		.ok_or_else(|| AppError::bad_request("Invalid or expired authorization code"))?;

	let (user_info, _) = state
		.user_cache
		.get(&entry.user_id)
		.await
		.ok_or_else(|| AppError::not_found("User data not found"))?;

	let expiration = Utc::now() + Duration::hours(24);
	let claims = Claims {
		sub: user_info.id.clone(),
		username: user_info.username.clone(),
		exp: expiration.timestamp() as usize,
	};

	let token = encode(&Header::default(), &claims, &state.jwt_encoding_key)
		.map_err(|e| AppError::internal(format!("Failed to generate JWT: {}", e)))?;

	debug!(user = %user_info.username, "issued jwt");

	Ok(Json(TokenExchangeResponse { token }))
}

async fn exchange_code_for_token(state: &AppState, code: &str) -> Result<TokenResponse, AppError> {
	let oauth_config = &state.config.api.oauth;

	let params = [
		("client_id", oauth_config.discord_client_id.as_str()),
		("client_secret", oauth_config.discord_client_secret.as_str()),
		("grant_type", "authorization_code"),
		("code", code),
		("redirect_uri", oauth_config.discord_redirect_uri.as_str()),
	];

	let response = state
		.http_client
		.post("https://discord.com/api/v10/oauth2/token")
		.form(&params)
		.send()
		.await
		.map_err(|e| AppError::bad_gateway(format!("discord api unreachable: {}", e)))?;

	if !response.status().is_success() {
		let status = response.status();
		let body = response.text().await.unwrap_or_default();
		error!(status = %status, body = %body, "discord token exchange rejected");
		return Err(AppError::bad_gateway(format!(
			"Token exchange failed with status {}",
			status
		)));
	}

	response.json::<TokenResponse>().await.map_err(|e| {
		error!(error = %e, "malformed discord token response");
		AppError::bad_gateway("Failed to parse Discord token response")
	})
}

pub async fn refresh_discord_token(
	state: &AppState, refresh_token: &str,
) -> Result<TokenResponse, AppError> {
	let oauth_config = &state.config.api.oauth;

	let params = [
		("client_id", oauth_config.discord_client_id.as_str()),
		("client_secret", oauth_config.discord_client_secret.as_str()),
		("grant_type", "refresh_token"),
		("refresh_token", refresh_token),
	];

	let response = state
		.http_client
		.post("https://discord.com/api/v10/oauth2/token")
		.form(&params)
		.send()
		.await
		.map_err(|e| AppError::bad_gateway(format!("discord api unreachable: {}", e)))?;

	if !response.status().is_success() {
		let status = response.status();
		let body = response.text().await.unwrap_or_default();
		error!(status = %status, body = %body, "discord token refresh rejected");
		return Err(AppError::bad_gateway(format!(
			"Token refresh failed with status {}",
			status
		)));
	}

	response.json::<TokenResponse>().await.map_err(|e| {
		error!(error = %e, "malformed discord refresh token response");
		AppError::bad_gateway("Failed to parse Discord token response")
	})
}

pub async fn get_user_info(
	client: &reqwest::Client, access_token: &str,
) -> Result<UserInfo, AppError> {
	let response = client
		.get("https://discord.com/api/v10/users/@me")
		.header("Authorization", format!("Bearer {}", access_token))
		.send()
		.await
		.map_err(|e| AppError::bad_gateway(format!("discord api unreachable: {}", e)))?;

	if !response.status().is_success() {
		let status = response.status();
		let body = response.text().await.unwrap_or_default();
		error!(status = %status, body = %body, "discord /users/@me failed");
		return Err(AppError::bad_gateway(format!(
			"Get user info failed with status {}",
			status
		)));
	}

	response.json::<UserInfo>().await.map_err(|e| {
		error!(error = %e, "malformed discord user info response");
		AppError::bad_gateway("Failed to parse user info")
	})
}

pub async fn get_user_guilds(
	client: &reqwest::Client, access_token: &str,
) -> Result<Vec<Guild>, AppError> {
	let response = client
		.get("https://discord.com/api/v10/users/@me/guilds")
		.header("Authorization", format!("Bearer {}", access_token))
		.send()
		.await
		.map_err(|e| AppError::bad_gateway(format!("discord api unreachable: {}", e)))?;

	if !response.status().is_success() {
		let status = response.status();
		let body = response.text().await.unwrap_or_default();
		error!(status = %status, body = %body, "discord /users/@me/guilds failed");
		return Err(AppError::bad_gateway(format!(
			"Get user guilds failed with status {}",
			status
		)));
	}

	let raw_guilds = response.json::<Vec<RawDiscordGuild>>().await.map_err(|e| {
		error!(error = %e, "malformed discord guilds response");
		AppError::bad_gateway("Failed to parse guilds")
	})?;

	Ok(raw_guilds
		.into_iter()
		.map(|raw_guild| {
			let icon_url = raw_guild.icon.as_ref().map(|icon_hash| {
				format!(
					"https://cdn.discordapp.com/icons/{}/{}.png",
					raw_guild.id, icon_hash
				)
			});
			Guild {
				id: raw_guild.id,
				name: raw_guild.name,
				icon_hash: raw_guild.icon,
				icon_url,
				owner: raw_guild.owner,
				permissions: raw_guild.permissions,
			}
		})
		.collect())
}

async fn store_oauth_tokens(
	state: &AppState, user_id: &str, token_response: &TokenResponse,
) -> Result<(), AppError> {
	let now = Utc::now().naive_utc();
	let expires_at = now + Duration::seconds(token_response.expires_in as i64);

	let existing = oauth_token::Entity::find_by_id(user_id.to_string())
		.one(&*state.db)
		.await?;

	if let Some(existing) = existing {
		let mut active: oauth_token::ActiveModel = existing.into();
		active.access_token = Set(token_response.access_token.clone());
		active.refresh_token = Set(token_response.refresh_token.clone());
		active.expires_at = Set(expires_at);
		active.updated_at = Set(now);
		active.update(&*state.db).await?;
	} else {
		let active = oauth_token::ActiveModel {
			user_id: Set(user_id.to_string()),
			access_token: Set(token_response.access_token.clone()),
			refresh_token: Set(token_response.refresh_token.clone()),
			expires_at: Set(expires_at),
			created_at: Set(now),
			updated_at: Set(now),
		};
		active.insert(&*state.db).await?;
	}

	debug!(user = %user_id, "persisted oauth tokens");
	Ok(())
}
