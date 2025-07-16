use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.create_table(
				Table::create()
					.table(Item::Table)
					.if_not_exists()
					.col(
						ColumnDef::new(Item::ItemId)
							.string()
							.not_null()
							.primary_key(),
					)
					.col(ColumnDef::new(Item::Name).string().not_null())
					.col(ColumnDef::new(Item::Description).string().not_null())
					.col(ColumnDef::new(Item::Price).integer().not_null())
					.col(ColumnDef::new(Item::MinimumRarity).integer().not_null())
					.col(ColumnDef::new(Item::MaximumRarity).integer().not_null())
					.col(ColumnDef::new(Item::MaxNumber).integer().not_null())
					.col(ColumnDef::new(Item::Type).string().not_null())
					.col(ColumnDef::new(Item::BaseXPBoost).float().not_null())
					.col(ColumnDef::new(Item::Weight).integer().not_null())
					.to_owned(),
			)
			.await
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.drop_table(Table::drop().table(Item::Table).to_owned())
			.await
	}
}

#[derive(DeriveIden)]
pub enum Item {
	Table,
	ItemId,
	Name,
	Description,
	Price,
	MinimumRarity,
	MaximumRarity,
	MaxNumber,
	Type,
	BaseXPBoost,
	Weight,
}
