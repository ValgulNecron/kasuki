use crate::event_handler::{BotData, Handler};
use crate::handlers::user_db::add_user_data_to_db;
use crate::server_image::calculate_user_color::{enqueue_user_color, get_member};
use crate::server_image::generate_server_image::{
	enqueue_global_server_image, enqueue_local_server_image, server_image_management,
};
use sea_orm::ActiveValue::Set;
use sea_orm::EntityTrait;
use serenity::all::{Guild, GuildMembersChunkEvent, Member};
use serenity::prelude::Context as SerenityContext;
use shared::database::prelude::{GuildData, ServerUserRelation, UserData};
use std::sync::atomic::Ordering;
use tracing::{info, trace, warn};

impl Handler {
	pub(crate) async fn guild_create(
		&self, ctx: &SerenityContext, guild: Guild, is_new: Option<bool>,
	) {
		let bot_data = ctx.data::<BotData>().clone();
		let image_config = bot_data.config.image.clone();
		let user_blacklist_server_image = bot_data.user_blacklist.clone();
		let db_connection = bot_data.db_connection.clone();

		let shard_info = ctx.runner_info.clone();
		let shard_id = ctx.shard_id.clone();
		// Read-then-write: avoid taking a write lock unless the shard is actually missing
		let data = bot_data.shard_manager.read().await;
		let shard_data = data.get(&shard_id);
		if let None = shard_data {
			// Must drop the read lock before acquiring write to avoid deadlock
			drop(data);
			let mut write = bot_data.shard_manager.write().await;
			write.insert(shard_id, shard_info);
		}

		if is_new.unwrap_or_default() {
			info!(guild_id = %guild.id, "Joined a new guild");

			let users = get_member(ctx, guild.id).await;
			for user in &users {
				enqueue_user_color(user_blacklist_server_image.clone(), user, &bot_data).await;
			}

			if let Err(e) =
				enqueue_local_server_image(ctx, guild.id, &image_config, db_connection.clone())
					.await
			{
				warn!(guild_id = %guild.id, error = %e, "Failed to enqueue local server image");
			}
			if let Err(e) =
				enqueue_global_server_image(ctx, guild.id, &image_config, db_connection.clone())
					.await
			{
				warn!(guild_id = %guild.id, error = %e, "Failed to enqueue global server image");
			}
		} else {
			info!(guild_id = %guild.id, "Guild already exists, skipping setup");
		}

		let active_guild = shared::database::guild_data::ActiveModel {
			guild_id: Set(guild.id.to_string()),
			guild_name: Set(guild.name.to_string()),
			updated_at: Set(guild.joined_at.naive_utc()),
		};

		// Upsert: insert new guild or update name/timestamp if it already exists
		if let Err(e) = GuildData::insert(active_guild)
			.on_conflict(
				sea_orm::sea_query::OnConflict::column(
					shared::database::guild_data::Column::GuildId,
				)
				.update_column(shared::database::guild_data::Column::GuildName)
				.update_column(shared::database::guild_data::Column::UpdatedAt)
				.to_owned(),
			)
			.exec(&*db_connection)
			.await
		{
			warn!(
				guild_id = %guild.id,
				error = %e,
				"Failed to insert or update guild data in database"
			);
		}
	}

	pub(crate) async fn guild_member_addition(&self, ctx: &SerenityContext, member: Member) {
		let bot_data = ctx.data::<BotData>().clone();
		let user_blacklist_server_image = bot_data.user_blacklist.clone();
		let guild_id = member.guild_id.to_string();
		let image_config = bot_data.config.image.clone();

		info!(
			user_id = %member.user.id,
			guild_id = %guild_id,
			"New member joined guild"
		);

		let user_id = member.user.id;

		enqueue_user_color(user_blacklist_server_image, &member.user, &bot_data).await;

		// Batch server image regeneration: only trigger after every 100 member joins
		let count = bot_data
			.user_color_update_count
			.fetch_add(1, Ordering::SeqCst);
		if count >= 100 {
			bot_data.user_color_update_count.store(0, Ordering::SeqCst);
			// Atomic swap acts as a lock-free mutex: only one regen task runs at a time
			if !bot_data.server_image_running.swap(true, Ordering::SeqCst) {
				let ctx_clone = ctx.clone();
				let image_config_clone = image_config.clone();
				let db_clone = bot_data.db_connection.clone();
				let flag = bot_data.server_image_running.clone();
				tokio::spawn(async move {
					server_image_management(&ctx_clone, image_config_clone, db_clone).await;
					// Release the "lock" so future triggers can start a new run
					flag.store(false, Ordering::SeqCst);
				});
			} else {
				info!("Server image generation already running, skipping");
			}
		}

		if let Err(e) = add_user_data_to_db(member.user, bot_data.db_connection.clone()).await {
			warn!(
				user_id = %user_id,
				error = ?e,
				"Failed to insert user data into database"
			);
		}
	}

	pub(crate) async fn guild_members_chunk(
		&self, ctx: &SerenityContext, chunk: GuildMembersChunkEvent,
	) {
		let bot_data = ctx.data::<BotData>().clone();
		let members = &chunk.members;

		if members.is_empty() {
			return;
		}
		trace!(
			guild_id = %chunk.guild_id,
			member_count = members.len(),
			"Received a chunk of guild members"
		);

		let guild_id_str = chunk.guild_id.to_string();

		let members_vec: Vec<&Member> = members.iter().collect();
		// Process in batches of 250 to stay within DB insert limits
		for batch in members_vec.chunks(250) {
			let user_models: Vec<shared::database::user_data::ActiveModel> = batch
				.iter()
				.map(|member| shared::database::user_data::ActiveModel {
					user_id: Set(member.user.id.to_string()),
					username: Set(member.user.name.to_string()),
					added_at: Set(chrono::Utc::now().naive_utc()),
					is_bot: Set(member.user.bot()),
				})
				.collect();

			// Upsert users: keep username/bot-flag current if user already exists
			if let Err(e) = UserData::insert_many(user_models)
				.on_conflict(
					sea_orm::sea_query::OnConflict::column(
						shared::database::user_data::Column::UserId,
					)
					.update_columns([
						shared::database::user_data::Column::Username,
						shared::database::user_data::Column::IsBot,
					])
					.to_owned(),
				)
				.exec(&*bot_data.db_connection)
				.await
			{
				warn!(
					guild_id = %chunk.guild_id,
					error = %e,
					"Failed to batch insert user data from chunk"
				);
			}

			let relation_models: Vec<shared::database::server_user_relation::ActiveModel> = batch
				.iter()
				.map(
					|member| shared::database::server_user_relation::ActiveModel {
						guild_id: Set(guild_id_str.clone()),
						user_id: Set(member.user.id.to_string()),
					},
				)
				.collect();

			// Silently skip duplicate guild-user pairs (composite PK conflict)
			if let Err(e) = ServerUserRelation::insert_many(relation_models)
				.on_conflict(
					sea_orm::sea_query::OnConflict::columns([
						shared::database::server_user_relation::Column::GuildId,
						shared::database::server_user_relation::Column::UserId,
					])
					.do_nothing()
					.to_owned(),
				)
				.exec(&*bot_data.db_connection)
				.await
			{
				match e {
					// RecordNotInserted is expected when do_nothing() skips duplicates
					sea_orm::DbErr::RecordNotInserted => {},
					_ => {
						warn!(
							guild_id = %chunk.guild_id,
							error = %e,
							"Failed to batch insert server-user relations from chunk"
						);
					},
				}
			}
		}
	}
}
