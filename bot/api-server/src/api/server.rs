use crate::api::auth::{auth_middleware, Claims};
use crate::api::oauth;
use crate::api::oauth::get_cached_user_data;
use axum::{
	http::StatusCode, middleware, response::IntoResponse, routing::get, Extension, Json, Router,
};
use serde::{Deserialize, Serialize};
use shared::config::Config;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

// This struct combines UserInfo and Guilds for the API response
#[derive(Debug, Serialize, Deserialize)]
pub struct UserDataResponse {
	pub user: User,
	pub guilds: Vec<Guild>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct User {
	pub id: String,
	pub username: String,
	pub avatar: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Guild {
	pub id: String,
	pub name: String,
	pub icon_url: Option<String>,
}

// Handler for the /api/user/:user_id endpoint
#[axum::debug_handler]
pub async fn get_user_profile(Extension(claims): Extension<Claims>) -> impl IntoResponse {
	let user_id = claims.sub; // Get user_id from JWT claims
	if let Some((user_info, guilds)) = get_cached_user_data(&user_id) {
		let user = User {
			id: user_info.id,
			username: user_info.username,
			avatar: user_info.avatar,
		};
		let guilds = guilds
			.into_iter()
			.map(|g| Guild {
				id: g.id,
				name: g.name,
				icon_url: g.icon_url,
			})
			.collect();

		let response = UserDataResponse { user, guilds };
		(StatusCode::OK, Json(response)).into_response()
	} else {
		(StatusCode::NOT_FOUND, "User not found".into_response()).into_response()
	}
}

pub async fn run_server(config: Arc<Config>) -> Result<(), Box<dyn std::error::Error>> {
	let port = config.api.port;

	// Configure CORS to allow requests from the frontend
	let cors = CorsLayer::new()
		.allow_origin(Any)
		.allow_methods(Any)
		.allow_headers(Any);

	let user_router =
		Router::new()
			.route("/me", get(get_user_profile))
			.layer(middleware::from_fn_with_state(
				config.clone(),
				auth_middleware,
			));

	// Build the router
	let app = Router::new()
		.route("/api/health", get(oauth::health_check))
		.route("/api/oauth/login", get(oauth::oauth_login))
		.route("/api/oauth/callback", get(oauth::oauth_callback))
		.nest("/api/user", user_router)
		.layer(cors)
		.with_state(config);

	// Create the server address
	let addr = format!("0.0.0.0:{}", port);
	info!("API server listening on {}", addr);

	// Start the server
	let listener = tokio::net::TcpListener::bind(&addr).await?;
	axum::serve(listener, app).await?;

	Ok(())
}
