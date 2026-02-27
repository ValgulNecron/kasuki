use crate::api::oauth::{Guild, UserInfo};
use moka::future::Cache;
use shared::config::Config;
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone)]
pub struct AuthCodeEntry {
	pub user_id: String,
}

#[derive(Clone)]
pub struct AppState {
	pub config: Arc<Config>,
	pub http_client: reqwest::Client,
	pub user_cache: Cache<String, (UserInfo, Vec<Guild>)>,
	pub auth_codes: Cache<String, AuthCodeEntry>,
	pub oauth_states: Cache<String, ()>,
	pub db: Arc<sea_orm::DatabaseConnection>,
	pub jwt_encoding_key: jsonwebtoken::EncodingKey,
	pub jwt_decoding_key: jsonwebtoken::DecodingKey,
}

impl AppState {
	pub fn new(
		config: Arc<Config>, db: sea_orm::DatabaseConnection, jwt_secret_bytes: Vec<u8>,
	) -> Self {
		let user_cache = Cache::builder()
			.max_capacity(10_000)
			.time_to_live(Duration::from_secs(24 * 60 * 60))
			.build();

		let auth_codes = Cache::builder()
			.max_capacity(10_000)
			.time_to_live(Duration::from_secs(5 * 60))
			.build();

		let oauth_states = Cache::builder()
			.max_capacity(10_000)
			.time_to_live(Duration::from_secs(10 * 60))
			.build();

		let jwt_encoding_key = jsonwebtoken::EncodingKey::from_secret(&jwt_secret_bytes);
		let jwt_decoding_key = jsonwebtoken::DecodingKey::from_secret(&jwt_secret_bytes);

		Self {
			config,
			http_client: reqwest::Client::new(),
			user_cache,
			auth_codes,
			oauth_states,
			db: Arc::new(db),
			jwt_encoding_key,
			jwt_decoding_key,
		}
	}
}
