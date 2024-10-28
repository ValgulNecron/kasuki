use crate::config::DbConfig;
use crate::structure::message::common::load_localization;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]

pub struct ModuleLocalised {
	pub on: String,
	pub off: String,
}

use anyhow::Result;

pub async fn load_localization_module_activation(
	guild_id: String, db_config: DbConfig,
) -> Result<ModuleLocalised> {
	let path = "json/message/admin/server/module.json";

	load_localization(guild_id, path, db_config).await
}
