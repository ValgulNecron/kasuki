use anyhow::{Context, Result};
use async_trait::async_trait;
use std::path::PathBuf;
use tracing::debug;

use crate::config::StorageConfig;

#[async_trait]
pub trait ImageStore: Send + Sync {
	async fn save(&self, key: &str, data: &[u8]) -> Result<()>;
	async fn load(&self, key: &str) -> Result<Vec<u8>>;
}

pub struct LocalImageStore {
	base_path: PathBuf,
}

impl LocalImageStore {
	pub fn new(base_path: impl Into<PathBuf>) -> Self {
		Self {
			base_path: base_path.into(),
		}
	}
}

#[async_trait]
impl ImageStore for LocalImageStore {
	async fn save(&self, key: &str, data: &[u8]) -> Result<()> {
		let path = self.base_path.join(key);
		if let Some(parent) = path.parent() {
			tokio::fs::create_dir_all(parent)
				.await
				.with_context(|| format!("Failed to create directory: {:?}", parent))?;
		}
		tokio::fs::write(&path, data)
			.await
			.with_context(|| format!("Failed to write file: {:?}", path))?;
		debug!("Saved image to local path: {:?}", path);
		Ok(())
	}

	async fn load(&self, key: &str) -> Result<Vec<u8>> {
		let path = self.base_path.join(key);
		let data = tokio::fs::read(&path)
			.await
			.with_context(|| format!("Failed to read file: {:?}", path))?;
		Ok(data)
	}
}

pub struct S3ImageStore {
	bucket: Box<s3::Bucket>,
}

impl S3ImageStore {
	pub fn new(
		bucket_name: &str, region: &str, endpoint: &str, access_key: &str, secret_key: &str,
	) -> Result<Self> {
		let region = s3::Region::Custom {
			region: region.into(),
			endpoint: endpoint.into(),
		};

		let credentials = s3::creds::Credentials::new(
			Some(access_key),
			Some(secret_key),
			None,
			None,
			None,
		)
		.context("Failed to create S3 credentials")?;

		let bucket = s3::Bucket::new(bucket_name, region, credentials)
			.context("Failed to create S3 bucket handle")?
			.with_path_style();

		Ok(Self { bucket })
	}
}

#[async_trait]
impl ImageStore for S3ImageStore {
	async fn save(&self, key: &str, data: &[u8]) -> Result<()> {
		self.bucket
			.put_object(key, data)
			.await
			.with_context(|| format!("Failed to upload to S3: {}", key))?;
		debug!("Saved image to S3: {}", key);
		Ok(())
	}

	async fn load(&self, key: &str) -> Result<Vec<u8>> {
		let response = self
			.bucket
			.get_object(key)
			.await
			.with_context(|| format!("Failed to download from S3: {}", key))?;
		Ok(response.to_vec())
	}
}

/// Create an `ImageStore` from config.
pub fn create_image_store(config: &StorageConfig) -> Result<Box<dyn ImageStore>> {
	match config.storage_type.as_str() {
		"local" => {
			let path = config
				.local_path
				.as_deref()
				.unwrap_or("./images");
			Ok(Box::new(LocalImageStore::new(path)))
		},
		"s3" => {
			let endpoint = config
				.s3_endpoint
				.as_deref()
				.context("s3_endpoint is required for S3 storage")?;
			let bucket = config
				.s3_bucket
				.as_deref()
				.context("s3_bucket is required for S3 storage")?;
			let region = config.s3_region.as_deref().unwrap_or("us-east-1");
			let access_key = config
				.s3_access_key
				.as_deref()
				.context("s3_access_key is required for S3 storage")?;
			let secret_key = config
				.s3_secret_key
				.as_deref()
				.context("s3_secret_key is required for S3 storage")?;
			Ok(Box::new(S3ImageStore::new(
				bucket, region, endpoint, access_key, secret_key,
			)?))
		},
		other => anyhow::bail!("Unknown storage_type: {}", other),
	}
}
