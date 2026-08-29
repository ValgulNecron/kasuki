use crate::api::oauth::{Guild, UserInfo};
use moka::future::Cache;
use serde::{Deserialize, Serialize};
use shared::cache::CacheInterface;
use shared::config::Config;
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone)]
pub struct AuthCodeEntry {
	pub user_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserCacheData {
	pub user: UserInfo,
	pub guilds: Vec<Guild>,
}

#[derive(Clone)]
pub struct AppState {
	pub config: Arc<Config>,
	pub http_client: reqwest::Client,
	pub user_cache: Arc<CacheInterface>,
	pub auth_codes: Cache<String, AuthCodeEntry>,
	pub oauth_states: Cache<String, ()>,
	pub db: Arc<sea_orm::DatabaseConnection>,
	pub jwt_encoding_key: jsonwebtoken::EncodingKey,
	pub jwt_decoding_key: jsonwebtoken::DecodingKey,
}

impl AppState {
	pub fn new(
		config: Arc<Config>, db: sea_orm::DatabaseConnection, jwt_secret_bytes: Vec<u8>,
		user_cache: Arc<CacheInterface>,
	) -> Self {
		let cache_cfg = &config.api.cache;

		let auth_codes = Cache::builder()
			.max_capacity(cache_cfg.auth_code_capacity)
			.time_to_live(Duration::from_secs(cache_cfg.auth_code_ttl_secs))
			.build();

		let oauth_states = Cache::builder()
			.max_capacity(cache_cfg.oauth_state_capacity)
			.time_to_live(Duration::from_secs(cache_cfg.oauth_state_ttl_secs))
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

	/// Write user data to the cache (serialized as JSON).
	pub async fn cache_user(&self, user_id: &str, user: &UserInfo, guilds: &[Guild]) {
		let data = UserCacheData {
			user: user.clone(),
			guilds: guilds.to_vec(),
		};
		match serde_json::to_string(&data) {
			Ok(json) => {
				if let Err(e) = self.user_cache.write(user_id.to_string(), json).await {
					tracing::warn!(user = %user_id, error = %e, "failed to write user data to cache");
				}
			},
			Err(e) => {
				tracing::warn!(user = %user_id, error = %e, "failed to serialize user data for cache");
			},
		}
	}

	/// Read user data from the cache.
	pub async fn get_cached_user(&self, user_id: &str) -> Option<(UserInfo, Vec<Guild>)> {
		let json = self.user_cache.read(user_id).await.ok().flatten()?;
		let data: UserCacheData = serde_json::from_str(&json).ok()?;
		Some((data.user, data.guilds))
	}
}
