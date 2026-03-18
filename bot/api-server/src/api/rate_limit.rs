use axum::{extract::ConnectInfo, http::Request, middleware::Next, response::Response};
use governor::{clock::DefaultClock, state::keyed::DashMapStateStore, Quota, RateLimiter};
use std::net::SocketAddr;
use std::num::NonZeroU32;
use std::sync::Arc;
use tracing::debug;

use crate::api::error::AppError;

pub type KeyedRateLimiter = RateLimiter<String, DashMapStateStore<String>, DefaultClock>;

pub fn create_rate_limiter(requests_per_minute: u32) -> Arc<KeyedRateLimiter> {
	let requests = NonZeroU32::new(requests_per_minute).expect("requests_per_minute must be > 0");
	Arc::new(RateLimiter::keyed(Quota::per_minute(requests)))
}

/// Spawns a background task that periodically evicts stale entries from the rate limiter.
pub fn spawn_rate_limiter_cleanup(limiter: &Arc<KeyedRateLimiter>) {
	let limiter = limiter.clone();
	tokio::spawn(async move {
		let mut interval = tokio::time::interval(std::time::Duration::from_secs(300));
		loop {
			interval.tick().await;
			limiter.retain_recent();
			limiter.shrink_to_fit();
			debug!("Rate limiter cleanup: {} active entries", limiter.len());
		}
	});
}

pub async fn rate_limit_middleware(
	ConnectInfo(addr): ConnectInfo<SocketAddr>,
	axum::extract::State(limiter): axum::extract::State<Arc<KeyedRateLimiter>>,
	req: Request<axum::body::Body>, next: Next,
) -> Result<Response, AppError> {
	let key = addr.ip().to_string();

	match limiter.check_key(&key) {
		Ok(_) => Ok(next.run(req).await),
		Err(_) => Err(AppError::rate_limited()),
	}
}
