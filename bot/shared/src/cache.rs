use std::hash::{DefaultHasher, Hash, Hasher};

use anyhow::{Context, Result};
use moka::future::Cache;
use redis::AsyncCommands;

use crate::config::CacheConfig;

enum CacheBackend {
	Memory(Cache<String, String>),
	Redis {
		connection: redis::aio::MultiplexedConnection,
		ttl_secs: u64,
	},
}

pub struct CacheInterface {
	backend: CacheBackend,
}

impl CacheInterface {
	/// In-memory cache with default settings (2 000 entries, 1 hour TTL).
	pub fn new() -> Self {
		Self::new_memory(2_000, 3600)
	}

	pub fn new_memory(max_capacity: u64, ttl_secs: u64) -> Self {
		let cache = Cache::builder()
			.max_capacity(max_capacity)
			.time_to_live(std::time::Duration::from_secs(ttl_secs))
			.build();
		Self {
			backend: CacheBackend::Memory(cache),
		}
	}

	pub fn new_redis(connection: redis::aio::MultiplexedConnection, ttl_secs: u64) -> Self {
		Self {
			backend: CacheBackend::Redis {
				connection,
				ttl_secs,
			},
		}
	}

	/// Build a `CacheInterface` from config. Returns an error only when
	/// `cache_type = "redis"` but the connection cannot be established;
	/// callers may fall back to `CacheInterface::new()` in that case.
	pub async fn from_config(config: &CacheConfig) -> Result<Self> {
		match config.cache_type.as_str() {
			"redis" => {
				let host = config.host.as_deref().unwrap_or("localhost");
				let port = config.port.unwrap_or(6379);
				let redis_url = match config.password.as_deref() {
					Some(pw) if !pw.is_empty() => {
						let encoded: String = pw
							.bytes()
							.map(|b| match b {
								b'A'..=b'Z'
								| b'a'..=b'z'
								| b'0'..=b'9'
								| b'-'
								| b'_'
								| b'.'
								| b'~' => String::from(b as char),
								_ => format!("%{:02X}", b),
							})
							.collect();
						format!("redis://:{}@{}:{}", encoded, host, port)
					},
					_ => format!("redis://{}:{}", host, port),
				};

				let client = redis::Client::open(redis_url.as_str())
					.context("Invalid Redis URL for cache")?;
				let conn = client
					.get_multiplexed_async_connection()
					.await
					.context("Failed to connect to Redis for cache")?;

				Ok(Self::new_redis(conn, config.ttl_secs))
			},
			_ => Ok(Self::new_memory(config.max_capacity, config.ttl_secs)),
		}
	}

	/// Create from config with a fallback to in-memory on failure.
	pub async fn from_config_or_default(config: &CacheConfig, label: &str) -> std::sync::Arc<Self> {
		std::sync::Arc::new(match Self::from_config(config).await {
			Ok(c) => {
				tracing::info!(
					"{} cache initialized with {} backend",
					label,
					config.cache_type
				);
				c
			},
			Err(e) => {
				tracing::warn!(
					"Failed to init {} cache with {} backend, falling back to memory: {}",
					label,
					config.cache_type,
					e
				);
				Self::new()
			},
		})
	}

	pub async fn read(&self, key: &str) -> Result<Option<String>> {
		let hashed = hash_key(key);
		match &self.backend {
			CacheBackend::Memory(cache) => Ok(cache.get(&hashed).await),
			CacheBackend::Redis { connection, .. } => {
				let mut conn = connection.clone();
				let value: Option<String> = conn.get(&hashed).await?;
				Ok(value)
			},
		}
	}

	pub async fn write(&self, key: String, value: String) -> Result<()> {
		let hashed = hash_key(&key);
		match &self.backend {
			CacheBackend::Memory(cache) => {
				cache.insert(hashed, value).await;
				Ok(())
			},
			CacheBackend::Redis {
				connection,
				ttl_secs,
			} => {
				let mut conn = connection.clone();
				conn.set_ex::<_, _, ()>(&hashed, &value, *ttl_secs).await?;
				Ok(())
			},
		}
	}
}

/// Hash a string to a fixed 32-char hex string (128-bit, two independent SipHash passes).
/// Collision probability for 10K entries: ~2.9×10⁻³¹.
pub fn hash_key(key: &str) -> String {
	let mut h1 = DefaultHasher::new();
	key.hash(&mut h1);
	let a = h1.finish();

	let mut h2 = DefaultHasher::new();
	0x517cc1b727220a95_u64.hash(&mut h2);
	key.hash(&mut h2);
	let b = h2.finish();

	format!("{:016x}{:016x}", a, b)
}

impl Default for CacheInterface {
	fn default() -> Self {
		Self::new()
	}
}
