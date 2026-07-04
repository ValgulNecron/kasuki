use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "user_inventory")]
pub struct Model {
	#[sea_orm(primary_key, auto_increment = false)]
	pub user_id: String,
	#[sea_orm(primary_key, auto_increment = false)]
	pub server_id: String,
	#[sea_orm(primary_key, auto_increment = false)]
	pub item_id: String,
	#[sea_orm(primary_key, auto_increment = false)]
	pub size: i32,
	#[sea_orm(primary_key, auto_increment = false)]
	pub rarity: i32,
	pub quantity: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
	#[sea_orm(
		belongs_to = "super::guild_data::Entity",
		from = "Column::ServerId",
		to = "super::guild_data::Column::GuildId",
		on_update = "Cascade",
		on_delete = "Cascade"
	)]
	GuildData,
	#[sea_orm(
		belongs_to = "super::item::Entity",
		from = "Column::ItemId",
		to = "super::item::Column::ItemId",
		on_update = "Cascade",
		on_delete = "Cascade"
	)]
	Item,
	#[sea_orm(
		belongs_to = "super::user_data::Entity",
		from = "Column::UserId",
		to = "super::user_data::Column::UserId",
		on_update = "Cascade",
		on_delete = "Cascade"
	)]
	UserData,
}

impl Related<super::guild_data::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::GuildData.def()
	}
}

impl Related<super::item::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::Item.def()
	}
}

impl Related<super::user_data::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::UserData.def()
	}
}

impl ActiveModelBehavior for ActiveModel {}
