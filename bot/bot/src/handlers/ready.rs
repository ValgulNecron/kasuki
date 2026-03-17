use crate::command::registry::get_slash_registry;
use crate::event_handler::{BotData, Handler};
use crate::helper::load_items::load_items_from_json;
use crate::launch_task::thread_management_launcher;
use crate::music_events;
use crate::register::registration_dispatcher::command_registration;
use crate::server_image::generate_server_image::server_image_management;
use lavalink_rs::model::events;
use lavalink_rs::node::NodeBuilder;
use lavalink_rs::prelude::NodeDistributionStrategy;
use sea_orm::ActiveValue::Set;
use sea_orm::EntityTrait;
use serenity::all::Ready;
use serenity::gateway::{ActivityData, ChunkGuildFilter};
use serenity::prelude::Context as SerenityContext;
use shared::database::prelude::CommandList;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tracing::{info, trace, warn};

impl Handler {
	pub(crate) async fn ready(&self, ctx: &SerenityContext, ready: Ready) {
		let bot_data = ctx.data::<BotData>().clone();
		if bot_data.lavalink.read().await.is_none() {
			if let Some(music_config) = bot_data.config.music.as_ref() {
				let events = events::Events {
					raw: Some(music_events::raw_event),
					ready: Some(music_events::ready_event),
					track_start: Some(music_events::track_start),
					..Default::default()
				};

				let user_id =
					lavalink_rs::model::UserId::from(ctx.cache.current_user().id.get());

				let node_local = NodeBuilder {
					hostname: music_config.lavalink_hostname.clone(),
					is_ssl: music_config.https,
					events: events::Events::default(),
					password: music_config.lavalink_password.clone(),
					user_id,
					session_id: None,
				};

				let client = lavalink_rs::client::LavalinkClient::new(
					events,
					vec![node_local],
					NodeDistributionStrategy::round_robin(),
				)
				.await;
				*bot_data.lavalink.write().await = Some(Arc::new(client));
				info!("Lavalink client initialized");
			} else {
				warn!("No music configuration found. Music features will be disabled.");
			}
		}

		let guilds = ctx.cache.guilds();
		info!("Requesting member chunks for {} guilds", guilds.len());
		for guild in &guilds {
			ctx.chunk_guild(*guild, None, true, ChunkGuildFilter::None, None);
			trace!(guild_id = %guild, "Chunking guild");

			tokio::time::sleep(std::time::Duration::from_millis(600)).await;
		}

		info!(
			"Shard {:?} of {} is connected!",
			ready.shard, ready.user.name
		);
		ctx.set_activity(Some(ActivityData::custom(
			bot_data.config.bot.bot_activity.clone(),
		)));

		if !*bot_data.already_launched.read().await {
			*bot_data.already_launched.write().await = true;
			tokio::spawn(thread_management_launcher(ctx.clone(), bot_data.clone()));

			let http = ctx.http.clone();
			let remove_old = bot_data.config.bot.remove_old_commands;
			let db_for_commands = bot_data.db_connection.clone();
			tokio::spawn(async move {
				command_registration(&http, remove_old).await;
				let registry = get_slash_registry();
				for command_name in registry.keys() {
					let model = shared::database::command_list::ActiveModel {
						command_name: Set(command_name.clone()),
					};
					if let Err(e) = CommandList::insert(model)
						.on_conflict(
							sea_orm::sea_query::OnConflict::column(
								shared::database::command_list::Column::CommandName,
							)
							.do_nothing()
							.to_owned(),
						)
						.exec(&*db_for_commands)
						.await
					{
						match e {
							sea_orm::DbErr::RecordNotInserted => {},
							_ => {
								warn!(command = %command_name, error = %e, "Failed to upsert command into command_list")
							},
						}
					}
				}
				info!(
					"Populated command_list table with {} commands",
					registry.len()
				);
			});

			let delay_secs = bot_data.config.task_intervals.before_server_image;
			if !bot_data.server_image_running.swap(true, Ordering::SeqCst) {
				let ctx_clone = ctx.clone();
				let image_config_clone = bot_data.config.image.clone();
				let db_clone = bot_data.db_connection.clone();
				let flag = bot_data.server_image_running.clone();
				tokio::spawn(async move {
					info!(
						"Waiting {}s before server image management to let cache populate",
						delay_secs
					);
					tokio::time::sleep(std::time::Duration::from_secs(delay_secs)).await;
					server_image_management(&ctx_clone, image_config_clone, db_clone).await;
					flag.store(false, Ordering::SeqCst);
				});
			}
		}

		let db = bot_data.db_connection.clone();
		if let Err(e) = load_items_from_json(&db).await {
			warn!(error = %e, "Failed to load items from JSON");
		}
	}
}
