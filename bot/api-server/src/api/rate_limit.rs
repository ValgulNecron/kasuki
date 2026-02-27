use axum::{
	extract::ConnectInfo,
	http::Request,
	middleware::Next,
	response::Response,
};
use governor::{
	Quota, RateLimiter,
	clock::DefaultClock,
	state::keyed::DashMapStateStore,
};
use std::net::SocketAddr;
use std::num::NonZeroU32;
use std::sync::Arc;

use crate::api::error::AppError;

pub type KeyedRateLimiter = RateLimiter<String, DashMapStateStore<String>, DefaultClock>;

pub fn create_rate_limiter(requests_per_minute: u32) -> Arc<KeyedRateLimiter> {
	Arc::new(RateLimiter::keyed(
		Quota::per_minute(NonZeroU32::new(requests_per_minute).expect("non-zero")),
	))
}

pub async fn rate_limit_middleware(
	ConnectInfo(addr): ConnectInfo<SocketAddr>,
	axum::extract::State(limiter): axum::extract::State<Arc<KeyedRateLimiter>>,
	req: Request<axum::body::Body>,
	next: Next,
) -> Result<Response, AppError> {
	let key = addr.ip().to_string();

	match limiter.check_key(&key) {
		Ok(_) => Ok(next.run(req).await),
		Err(_) => Err(AppError::rate_limited()),
	}
}
