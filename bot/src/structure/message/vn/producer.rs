use crate::config::DbConfig;
use crate::structure::message::common::load_localization;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]

pub struct ProducerLocalised {
	pub lang: String,

	pub prod_type: String,
	pub aliases: String,
}

use anyhow::Result;

pub async fn load_localization_producer(
	guild_id: String, db_config: DbConfig,
) -> Result<ProducerLocalised> {
	let path = "json/message/vn/producer.json";

	load_localization(guild_id, path, db_config).await
}
