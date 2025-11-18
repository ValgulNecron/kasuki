//! `SeaORM` Entity for user_session table

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "user_session")]
pub struct Model {
	#[sea_orm(primary_key, auto_increment = false)]
	pub session_token: String,
	pub user_id: String,
	pub discord_access_token: String,
	pub discord_refresh_token: String,
	pub token_expires_at: DateTime,
	pub created_at: DateTime,
	pub last_used_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
