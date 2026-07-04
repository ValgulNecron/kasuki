//! Collapse `user_inventory` from one-row-per-catch to one-row-per-stack.
//!
//! Every fishing catch used to insert a new UUID-keyed row, which made the
//! table grow unbounded for active players. Identical specimens (same item,
//! size, rarity) now stack: one row per `(user_id, server_id, item_id, size,
//! rarity)` with a `quantity` counter. The redundant `item_xp_boost` column
//! is dropped too, since it was always equal to `item.base_xp_boost` for the
//! same item.

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		let db = manager.get_connection();

		db.execute_unprepared(
			"ALTER TABLE user_inventory \
			 ADD COLUMN IF NOT EXISTS quantity bigint NOT NULL DEFAULT 1",
		)
		.await?;

		// Set quantity = group count on the kept row (lowest id per group),
		// then drop the duplicates.
		db.execute_unprepared(
			"UPDATE user_inventory ui \
			 SET quantity = g.cnt \
			 FROM ( \
			     SELECT MIN(id) AS keep_id, COUNT(*) AS cnt \
			     FROM user_inventory \
			     GROUP BY user_id, server_id, item_id, size, rarity \
			 ) g \
			 WHERE ui.id = g.keep_id",
		)
		.await?;

		db.execute_unprepared(
			"DELETE FROM user_inventory \
			 WHERE id NOT IN ( \
			     SELECT MIN(id) FROM user_inventory \
			     GROUP BY user_id, server_id, item_id, size, rarity \
			 )",
		)
		.await?;

		// Replace the PK so future inserts can hit the conflict path cleanly.
		db.execute_unprepared("ALTER TABLE user_inventory DROP CONSTRAINT user_inventory_pkey")
			.await?;
		db.execute_unprepared("ALTER TABLE user_inventory DROP COLUMN id")
			.await?;
		db.execute_unprepared("ALTER TABLE user_inventory DROP COLUMN item_xp_boost")
			.await?;
		db.execute_unprepared(
			"ALTER TABLE user_inventory \
			 ADD PRIMARY KEY (user_id, server_id, item_id, size, rarity)",
		)
		.await?;

		Ok(())
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		let db = manager.get_connection();

		db.execute_unprepared("ALTER TABLE user_inventory DROP CONSTRAINT user_inventory_pkey")
			.await?;
		db.execute_unprepared(
			"ALTER TABLE user_inventory ADD COLUMN id text NOT NULL DEFAULT gen_random_uuid()::text",
		)
		.await?;
		db.execute_unprepared(
			"ALTER TABLE user_inventory ADD COLUMN item_xp_boost real NOT NULL DEFAULT 0",
		)
		.await?;
		db.execute_unprepared(
			"ALTER TABLE user_inventory ADD PRIMARY KEY (id, item_id, user_id, server_id)",
		)
		.await?;
		db.execute_unprepared("ALTER TABLE user_inventory DROP COLUMN quantity")
			.await?;

		Ok(())
	}
}
