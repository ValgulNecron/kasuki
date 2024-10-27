use crate::config::DbConfig;
use crate::structure::message::common::load_localization;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]

pub struct CreditLocalisedLine {
	pub desc: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]

pub struct CreditLocalised {
	pub title: String,
	pub credits: Vec<CreditLocalisedLine>,
}

use anyhow::Result;

pub async fn load_localization_credit(
	guild_id: String, db_config: DbConfig,
) -> Result<CreditLocalised> {
	let path = "json/message/bot/credit.json";

	load_localization(guild_id, path, db_config).await
}
