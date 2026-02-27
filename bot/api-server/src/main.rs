mod api;

use api::state::AppState;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use shared::config::Config;
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	tracing_subscriber::fmt::init();
	info!("starting api-server");

	let config = Config::new().map_err(|e| {
		error!(error = %e, "failed to load config");
		e
	})?;

	let jwt_secret_bytes = STANDARD
		.decode(&config.api.oauth.jwt_secret)
		.map_err(|e| {
			error!(error = %e, "invalid base64 jwt secret in config");
			anyhow::anyhow!("Invalid base64 JWT secret: {}", e)
		})?;
	info!("jwt secret validated");

	let config = Arc::new(config);

	let db_url = get_db_url(&config);
	let mut connect_options = sea_orm::ConnectOptions::new(db_url);

	let max_connections = config.db.max_connections.unwrap_or(100);
	let min_connections = config.db.min_connections.unwrap_or(5);
	let connect_timeout = config.db.connect_timeout.unwrap_or(30);
	let idle_timeout = config.db.idle_timeout.unwrap_or(600);

	connect_options
		.max_connections(max_connections)
		.min_connections(min_connections)
		.connect_timeout(Duration::from_secs(connect_timeout))
		.idle_timeout(Duration::from_secs(idle_timeout))
		.sqlx_logging(false);

	info!(
		max = max_connections,
		min = min_connections,
		connect_timeout_s = connect_timeout,
		idle_timeout_s = idle_timeout,
		"database pool configured"
	);

	let db = sea_orm::Database::connect(connect_options).await.map_err(|e| {
		error!(error = %e, "failed to connect to database");
		anyhow::anyhow!("Database connection failed: {}", e)
	})?;
	info!("database connected");

	let state = AppState::new(config, db, jwt_secret_bytes);

	api::start_api_server(state).await;

	Ok(())
}

fn get_db_url(config: &Config) -> String {
	match config.db.db_type.as_str() {
		"postgresql" => {
			let host = config.db.host.as_deref().unwrap_or("localhost");
			let port = config.db.port.unwrap_or(5432);
			let user = config.db.user.as_deref().unwrap_or("postgres");
			let password = config.db.password.as_deref().unwrap_or("");
			let database = config.db.database.as_deref().unwrap_or("kasuki");
			format!(
				"postgres://{}:{}@{}:{}/{}",
				user, password, host, port, database
			)
		},
		_ => "sqlite://kasuki.db?mode=rwc".to_string(),
	}
}
