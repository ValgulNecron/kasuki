use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.create_table(
				Table::create()
					.table(OAuthToken::Table)
					.if_not_exists()
					.col(string(OAuthToken::UserId))
					.primary_key(Index::create().col(OAuthToken::UserId))
					.col(string(OAuthToken::AccessToken))
					.col(string(OAuthToken::RefreshToken))
					.col(timestamp(OAuthToken::ExpiresAt))
					.col(timestamp(OAuthToken::CreatedAt).default(Expr::current_timestamp()))
					.col(timestamp(OAuthToken::UpdatedAt).default(Expr::current_timestamp()))
					.to_owned(),
			)
			.await
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.drop_table(Table::drop().table(OAuthToken::Table).to_owned())
			.await
	}
}

#[derive(DeriveIden)]
pub enum OAuthToken {
	Table,
	UserId,
	AccessToken,
	RefreshToken,
	ExpiresAt,
	CreatedAt,
	UpdatedAt,
}
