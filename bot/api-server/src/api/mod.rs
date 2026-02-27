pub mod auth;
pub mod error;
pub mod health;
pub mod oauth;
pub mod rate_limit;
pub mod server;
pub mod state;
#[cfg(test)]
mod tests;

use crate::api::state::AppState;
use tracing::info;

pub async fn start_api_server(state: AppState) {
	if !state.config.api.enabled {
		info!("api server disabled in config");
		return;
	}

	info!(port = state.config.api.port, "starting api server");

	if let Err(e) = server::run_server(state).await {
		tracing::error!(error = %e, "api server stopped with error");
	}
}
