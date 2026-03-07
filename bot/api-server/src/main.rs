mod api;

use api::state::AppState;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use shared::config::Config;
use std::sync::Arc;
use tracing::{error, info};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	let config = Config::new().map_err(|e| {
		eprintln!("failed to load config: {}", e);
		e
	})?;

	let _sentry_guard = config.sentry_url.as_deref().map(|url| {
		let guard = sentry::init((
			url,
			sentry::ClientOptions {
				release: sentry::release_name!(),
				..Default::default()
			},
		));
		println!("Sentry initialized successfully");
		guard
	});

	let sentry_layer = sentry::integrations::tracing::layer();
	tracing_subscriber::registry()
		.with(sentry_layer)
		.with(tracing_subscriber::fmt::layer())
		.init();
	info!("starting api-server");

	let jwt_secret_bytes = STANDARD.decode(&config.api.oauth.jwt_secret).map_err(|e| {
		error!(error = %e, "invalid base64 jwt secret in config");
		anyhow::anyhow!("Invalid base64 JWT secret: {}", e)
	})?;
	info!("jwt secret validated");

	let config = Arc::new(config);

	let db = config.db.connect().await.map_err(|e| {
		error!(error = %e, "failed to connect to database");
		e
	})?;
	info!("database connected");

	let state = AppState::new(config, db, jwt_secret_bytes);

	api::start_api_server(state).await;

	Ok(())
}
