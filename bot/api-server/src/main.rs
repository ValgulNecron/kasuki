mod api;

use shared::config::Config;
use std::sync::Arc;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	tracing_subscriber::fmt::init();
	info!("Starting API Server...");

	let config = Config::new().map_err(|e| {
		tracing::error!("Failed to load config: {}", e);
		e
	})?;

	let config = Arc::new(config);

	api::start_api_server(config).await;

	Ok(())
}
