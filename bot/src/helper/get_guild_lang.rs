use crate::config::DbConfig;
use crate::database::guild_lang::{Column, Model};
use crate::database::prelude::GuildLang;
use crate::get_url;
use sea_orm::ColumnTrait;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;

pub async fn get_guild_language(guild_id: String, db_config: DbConfig) -> String {
	if guild_id == *"0" {
		return String::from("en");
	};

	let connection = match sea_orm::Database::connect(get_url(db_config.clone())).await {
		Ok(conn) => conn,
		Err(_) => {
			return String::from("en");
		},
	};

	let guild_lang: Option<Model> = GuildLang::find()
		.filter(Column::GuildId.eq(guild_id))
		.one(&connection)
		.await
		.unwrap_or(None);

	match guild_lang {
		Some(lang) => lang.lang,
		None => String::from("en"),
	}
}
