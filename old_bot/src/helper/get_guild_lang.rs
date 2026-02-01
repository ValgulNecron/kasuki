use crate::database::guild_lang::{Column, Model};
use crate::database::prelude::GuildLang;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use sea_orm::{ColumnTrait, DatabaseConnection};
use std::sync::Arc;

pub async fn get_guild_language(
    guild_id: String, db_connection: Arc<DatabaseConnection>,
) -> String {
    if guild_id == *"0" {
        return String::from("en");
    };

    let guild_lang: Option<Model> = GuildLang::find()
        .filter(Column::GuildId.eq(guild_id))
        .one(&*db_connection)
        .await
        .unwrap_or(None);

    match guild_lang {
        Some(lang) => lang.lang,
        None => String::from("en"),
    }
}
