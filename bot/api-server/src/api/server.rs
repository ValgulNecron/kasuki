use crate::api::auth::{auth_middleware, Claims};
use crate::api::blacklist;
use crate::api::error::AppError;
use crate::api::guild;
use crate::api::oauth::{get_user_guilds, get_user_info, refresh_discord_token, Guild, UserInfo};
use crate::api::rate_limit::{
	create_rate_limiter, rate_limit_middleware, spawn_rate_limiter_cleanup,
};
use crate::api::state::AppState;
use crate::api::{health, oauth as oauth_handlers};
use axum::{
	extract::State,
	http::Method,
	middleware,
	response::IntoResponse,
	routing::{get, post},
	Extension, Json, Router,
};
use chrono::{Duration, Utc};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, EntityTrait};
use serde::{Deserialize, Serialize};
use shared::database::oauth_token;
use tower_http::cors::{AllowOrigin, Any, CorsLayer};
use tracing::{debug, error, info};

#[derive(Debug, Serialize, Deserialize)]
pub struct UserDataResponse {
	pub user: UserInfo,
	pub guilds: Vec<Guild>,
}

#[axum::debug_handler]
pub async fn get_user_profile(
	State(state): State<AppState>, Extension(claims): Extension<Claims>,
) -> Result<impl IntoResponse, AppError> {
	let user_id = claims.sub;

	let (user_info, guilds) = state
		.get_cached_user(&user_id)
		.await
		.ok_or_else(|| AppError::not_found("User not found"))?;

	Ok(Json(UserDataResponse {
		user: user_info,
		guilds,
	}))
}

#[axum::debug_handler]
pub async fn update_user_data(
	State(state): State<AppState>, Extension(claims): Extension<Claims>,
) -> Result<Json<UserDataResponse>, AppError> {
	let user_id = &claims.sub;

	let token_record = oauth_token::Entity::find_by_id(user_id.clone())
		.one(&*state.db)
		.await?
		.ok_or_else(|| AppError::not_found("No stored tokens found"))?;

	let access_token = if token_record.expires_at < Utc::now().naive_utc() {
		debug!(user = %user_id, "discord token expired, refreshing");
		let new_tokens = refresh_discord_token(&state, &token_record.refresh_token).await?;

		let now = Utc::now().naive_utc();
		let expires_at = now + Duration::seconds(new_tokens.expires_in as i64);
		let mut active: oauth_token::ActiveModel = token_record.into();
		active.access_token = Set(new_tokens.access_token.clone());
		active.refresh_token = Set(new_tokens.refresh_token.clone());
		active.expires_at = Set(expires_at);
		active.updated_at = Set(now);
		active.update(&*state.db).await?;

		new_tokens.access_token
	} else {
		token_record.access_token
	};

	let user_info = get_user_info(&state.http_client, &access_token).await?;
	let guilds = get_user_guilds(&state.http_client, &access_token).await?;

	state.cache_user(user_id, &user_info, &guilds).await;

	info!(user = %user_id, "refreshed user data from discord");

	Ok(Json(UserDataResponse {
		user: user_info,
		guilds,
	}))
}

fn build_cors_layer(config: &shared::config::Config) -> CorsLayer {
	if config.api.debug {
		CorsLayer::new()
			.allow_origin(Any)
			.allow_methods(Any)
			.allow_headers(Any)
	} else if let Some(ref domain) = config.api.allowed_domain {
		let domain = domain.clone();
		let frontend_url = config.api.oauth.frontend_url.clone();
		CorsLayer::new()
			.allow_origin(AllowOrigin::predicate(move |origin, _| {
				let origin_str = origin.to_str().unwrap_or("");
				if origin_str == frontend_url {
					return true;
				}
				if let Some(host_part) = origin_str.split("://").nth(1) {
					let host = host_part.split(':').next().unwrap_or(host_part);
					return host == domain || host.ends_with(&format!(".{}", domain));
				}
				false
			}))
			.allow_methods([
				Method::GET,
				Method::POST,
				Method::PUT,
				Method::DELETE,
				Method::OPTIONS,
			])
			.allow_headers([
				axum::http::header::AUTHORIZATION,
				axum::http::header::CONTENT_TYPE,
			])
			.allow_credentials(true)
	} else {
		let frontend_url = config.api.oauth.frontend_url.clone();
		CorsLayer::new()
			.allow_origin(AllowOrigin::predicate(move |origin, _| {
				origin.to_str().unwrap_or("") == frontend_url
			}))
			.allow_methods([
				Method::GET,
				Method::POST,
				Method::PUT,
				Method::DELETE,
				Method::OPTIONS,
			])
			.allow_headers([
				axum::http::header::AUTHORIZATION,
				axum::http::header::CONTENT_TYPE,
			])
			.allow_credentials(true)
	}
}

/// Background task: refresh all Discord OAuth tokens that expire within the next hour.
/// Runs every 30 minutes so tokens are always fresh, even if the user never comes back.
pub fn spawn_discord_token_refresh(state: AppState) {
	tokio::spawn(async move {
		let mut interval = tokio::time::interval(std::time::Duration::from_secs(30 * 60));
		loop {
			interval.tick().await;
			if let Err(e) = refresh_expiring_tokens(&state).await {
				error!("discord token refresh task failed: {}", e.message);
			}
		}
	});
}

async fn refresh_expiring_tokens(state: &AppState) -> Result<(), AppError> {
	use sea_orm::{ColumnTrait, QueryFilter};

	// Find tokens expiring within the next hour
	let cutoff = (Utc::now() + Duration::hours(1)).naive_utc();
	let expiring = oauth_token::Entity::find()
		.filter(oauth_token::Column::ExpiresAt.lt(cutoff))
		.all(&*state.db)
		.await?;

	if expiring.is_empty() {
		return Ok(());
	}

	info!(
		count = expiring.len(),
		"refreshing discord tokens expiring soon"
	);

	for record in expiring {
		let user_id = record.user_id.clone();
		match refresh_discord_token(state, &record.refresh_token).await {
			Ok(new_tokens) => {
				let now = Utc::now().naive_utc();
				let expires_at = now + Duration::seconds(new_tokens.expires_in as i64);
				let mut active: oauth_token::ActiveModel = record.into();
				active.access_token = Set(new_tokens.access_token);
				active.refresh_token = Set(new_tokens.refresh_token);
				active.expires_at = Set(expires_at);
				active.updated_at = Set(now);
				if let Err(e) = active.update(&*state.db).await {
					error!(user = %user_id, error = %e, "failed to save refreshed tokens");
				} else {
					debug!(user = %user_id, "refreshed discord token");
				}
			},
			Err(e) => {
				// Refresh token might be revoked — delete the record so it doesn't retry forever
				error!(user = %user_id, error = %e.message, "discord token refresh failed, removing record");
				let _ = oauth_token::Entity::delete_by_id(user_id)
					.exec(&*state.db)
					.await;
			},
		}
	}

	Ok(())
}

pub async fn run_server(state: AppState) -> anyhow::Result<()> {
	let port = state.config.api.port;
	let cors = build_cors_layer(&state.config);

	let rate_limiter = create_rate_limiter(state.config.api.rate_limit_per_minute);
	spawn_rate_limiter_cleanup(&rate_limiter);

	// Background task to keep Discord tokens fresh
	spawn_discord_token_refresh(state.clone());

	let oauth_router = Router::new()
		.route("/login", get(oauth_handlers::oauth_login))
		.route("/callback", get(oauth_handlers::oauth_callback))
		.route("/token", post(oauth_handlers::exchange_auth_code))
		.layer(middleware::from_fn_with_state(
			rate_limiter.clone(),
			rate_limit_middleware,
		))
		.with_state(state.clone());

	let user_router = Router::new()
		.route("/me", get(get_user_profile))
		.route("/update", post(update_user_data))
		.route("/blacklist-request", post(blacklist::request_blacklist))
		.layer(middleware::from_fn_with_state(
			state.clone(),
			auth_middleware,
		))
		.with_state(state.clone());

	let guild_router = guild::guild_router(state.clone());
	let anilist_router = guild::anilist_router(state.clone());

	let app = Router::new()
		.route("/api/health", get(health::health_check))
		.route(
			"/api/health/ready",
			get(health::readiness_check).with_state(state.clone()),
		)
		.nest("/api/oauth", oauth_router)
		.nest("/api/user", user_router)
		.nest("/api/guild", guild_router)
		.nest("/api/anilist", anilist_router)
		.layer(cors);

	let addr = format!("0.0.0.0:{}", port);
	info!(addr = %addr, "api server listening");

	let listener = tokio::net::TcpListener::bind(&addr).await?;
	let make_service = app.into_make_service_with_connect_info::<std::net::SocketAddr>();
	axum::serve(listener, make_service).await?;

	Ok(())
}
