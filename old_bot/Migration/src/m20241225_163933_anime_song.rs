use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.create_table(
				Table::create()
					.table(AnimeSong::Table)
					.if_not_exists()
					.col(string(AnimeSong::AnilistId))
					.col(string(AnimeSong::AnnId))
					.col(string(AnimeSong::AnnSongId))
					.col(string(AnimeSong::AnimeEnName))
					.col(string(AnimeSong::AnimeJpName))
					.col(string(AnimeSong::AnimeAltName))
					.col(string(AnimeSong::SongType))
					.col(string(AnimeSong::SongName))
					.col(string(AnimeSong::Hq))
					.col(string(AnimeSong::Mq))
					.col(string(AnimeSong::Audio))
					.primary_key(
						Index::create()
							.col(AnimeSong::AnilistId)
							.col(AnimeSong::AnnId)
							.col(AnimeSong::AnnSongId),
					)
					.to_owned(),
			)
			.await
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.drop_table(Table::drop().table(AnimeSong::Table).to_owned())
			.await
	}
}

#[derive(DeriveIden)]
enum AnimeSong {
	Table,
	AnilistId,
	AnnId,
	AnnSongId,
	AnimeEnName,
	AnimeJpName,
	AnimeAltName,
	SongType,
	SongName,
	Hq,
	Mq,
	Audio,
}
