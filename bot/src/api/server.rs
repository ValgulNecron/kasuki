use crate::api::oauth;
use crate::config::Config;
use axum::{
	routing::get,
	Router,
};
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

#[derive(Clone)]
pub struct ApiState {
	pub config: Arc<Config>,
	pub db: DatabaseConnection,
}

pub async fn run_server(config: Arc<Config>, db: DatabaseConnection) -> Result<(), Box<dyn std::error::Error>> {
	let port = config.api.port;

	let state = ApiState {
		config: config.clone(),
		db,
	};

	// Configure CORS to allow requests from the frontend
	let cors = CorsLayer::new()
		.allow_origin(Any)
		.allow_methods(Any)
		.allow_headers(Any);

	// Build the router
	let app = Router::new()
		.route("/api/health", get(oauth::health_check))
		.route("/api/oauth/login", get(oauth::oauth_login))
		.route("/api/oauth/callback", get(oauth::oauth_callback))
		.route("/api/session/validate", get(oauth::validate_session))
		.route("/api/session/logout", get(oauth::logout))
		.layer(cors)
		.with_state(state);

	// Create the server address
	let addr = format!("0.0.0.0:{}", port);
	info!("API server listening on {}", addr);

	// Start the server
	let listener = tokio::net::TcpListener::bind(&addr).await?;
	axum::serve(listener, app).await?;

	Ok(())
}
