use crate::structure::message::common::load_localization;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Deserialize, Serialize, Clone)]

pub struct BannerLocalised {
	pub title: String,
	pub no_banner: String,
	pub no_banner_title: String,
}

use anyhow::Result;
use sea_orm::DatabaseConnection;

pub async fn load_localization_banner(
	guild_id: String, db_connection: Arc<DatabaseConnection>,
) -> Result<BannerLocalised> {
	let path = "json/message/user/banner.json";

	load_localization(guild_id, path, db_connection).await
}
