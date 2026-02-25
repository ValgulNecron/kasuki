use crate::event_handler::{BotData, Handler};
use crate::handlers::user_db::add_user_data_to_db;
use crate::server_image::calculate_user_color::{color_management, get_specific_user_color};
use crate::server_image::generate_server_image::server_image_management;
use sea_orm::ActiveValue::Set;
use sea_orm::EntityTrait;
use serenity::all::{Guild, GuildMembersChunkEvent, Member};
use serenity::prelude::Context as SerenityContext;
use shared::database::prelude::{GuildData, ServerUserRelation};
use std::sync::atomic::Ordering;
use tracing::{info, trace, warn};

impl Handler {
	pub(crate) async fn guild_create(
		&self, ctx: SerenityContext, guild: Guild, is_new: Option<bool>,
	) {
		let bot_data = ctx.data::<BotData>().clone();
		let image_config = bot_data.config.image.clone();
		let user_blacklist_server_image = bot_data.user_blacklist.clone();
		let db_connection = bot_data.db_connection.clone();

		if is_new.unwrap_or_default() {
			info!(guild_id = %guild.id, "Joined a new guild");
			color_management(
				&ctx.cache.guilds(),
				&ctx,
				user_blacklist_server_image,
				bot_data.clone(),
			)
			.await;
			if !bot_data.server_image_running.swap(true, Ordering::SeqCst) {
				let flag = bot_data.server_image_running.clone();
				server_image_management(&ctx, image_config, db_connection.clone()).await;
				flag.store(false, Ordering::SeqCst);
			} else {
				info!(guild_id = %guild.id, "Server image generation already running, skipping");
			}
		} else {
			info!(guild_id = %guild.id, "Guild already exists, skipping setup");
		}

		let active_guild = shared::database::guild_data::ActiveModel {
			guild_id: Set(guild.id.to_string()),
			guild_name: Set(guild.name.to_string()),
			updated_at: Set(guild.joined_at.naive_utc()),
		};

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

	pub(crate) async fn guild_member_addition(&self, ctx: SerenityContext, member: Member) {
		let bot_data = ctx.data::<BotData>().clone();
		let user_blacklist_server_image = bot_data.user_blacklist.clone();
		let guild_id = member.guild_id.to_string();
		let image_config = bot_data.config.image.clone();

		info!(
			user_id = %member.user.id,
			guild_id = %guild_id,
			"New member joined guild"
		);

		get_specific_user_color(
			user_blacklist_server_image,
			member.user.clone(),
			bot_data.db_connection.clone(),
		)
		.await;

		let count = bot_data
			.user_color_update_count
			.fetch_add(1, Ordering::SeqCst);
		if count >= 100 {
			bot_data.user_color_update_count.store(0, Ordering::SeqCst);
			if !bot_data.server_image_running.swap(true, Ordering::SeqCst) {
				let ctx_clone = ctx.clone();
				let image_config_clone = image_config.clone();
				let db_clone = bot_data.db_connection.clone();
				let flag = bot_data.server_image_running.clone();
				tokio::spawn(async move {
					server_image_management(&ctx_clone, image_config_clone, db_clone).await;
					flag.store(false, Ordering::SeqCst);
				});
			} else {
				info!("Server image generation already running, skipping");
			}
		}

		let user = member.user.clone();

		if let Err(e) = add_user_data_to_db(user, bot_data.db_connection.clone()).await {
			warn!(
				user_id = %member.user.id,
				error = ?e,
				"Failed to insert user data into database"
			);
		}
	}

	pub(crate) async fn guild_members_chunk(
		&self, ctx: SerenityContext, chunk: GuildMembersChunkEvent,
	) {
		let bot_data = ctx.data::<BotData>().clone();
		let members = &chunk.members;

		if members.is_empty() {
			return;
		}
		trace!(
			guild_id = %chunk.guild_id,
			"Received a chunk of guild members"
		);

		let db_connection = bot_data.db_connection.clone();

		for member in members {
			let user = member.user.clone();

			if let Err(e) = add_user_data_to_db(user.clone(), db_connection.clone()).await {
				warn!(
					user_id = %user.id,
					error = ?e,
					"Failed to insert user data from chunk into database"
				);
			}

			let active_relation = shared::database::server_user_relation::ActiveModel {
				guild_id: Set(chunk.guild_id.to_string()),
				user_id: Set(user.id.to_string()),
			};

			if let Err(e) = ServerUserRelation::insert(active_relation)
				.on_conflict(
					sea_orm::sea_query::OnConflict::columns([
						shared::database::server_user_relation::Column::GuildId,
						shared::database::server_user_relation::Column::UserId,
					])
					.do_nothing()
					.to_owned(),
				)
				.exec(&*db_connection.clone())
				.await
			{
				match e {
					sea_orm::DbErr::RecordNotInserted => {},
					_ => {
						warn!(
							user_id = %user.id,
							guild_id = %chunk.guild_id,
							error = %e,
							"Failed to insert server-user relation from chunk into database"
						);
					},
				}
			}
		}
	}
}
