pub mod oauth;
pub mod server;

use crate::config::Config;
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use tracing::info;

pub async fn start_api_server(config: Arc<Config>, db: DatabaseConnection) {
	if !config.api.enabled {
		info!("API server is disabled in configuration");
		return;
	}

	info!(
		"Starting API server on port {}",
		config.api.port
	);

	if let Err(e) = server::run_server(config, db).await {
		tracing::error!("API server error: {}", e);
	}
}
