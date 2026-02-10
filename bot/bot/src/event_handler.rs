use crate::autocomplete::autocomplete_dispatch::autocomplete_dispatching;
use crate::command::command_dispatch::{check_if_module_is_on, dispatch_command};
use crate::command::user_command_dispatch::dispatch_user_command;
use crate::components::components_dispatch::components_dispatching;
use crate::error_management::error_dispatch;
use crate::register::registration_dispatcher::command_registration;
use crate::server_image::calculate_user_color::{color_management, get_specific_user_color};
use crate::server_image::generate_server_image::server_image_management;
use chrono::{DateTime, Timelike, Utc};
use num_bigint::BigUint;
use reqwest::Client;
use sea_orm::ActiveValue::Set;
use sea_orm::{DatabaseConnection, EntityTrait};
use serde::{Deserialize, Serialize};
use serenity::all::{
	CommandType, CurrentApplicationInfo, Entitlement, Guild, GuildMembersChunkEvent, Interaction,
	Member, Message, Presence, Ready, ShardId, User,
};
use serenity::all::{FullEvent, VoiceState};
use serenity::async_trait;
use serenity::gateway::{ActivityData, ChunkGuildFilter, ShardRunnerInfo, ShardRunnerMessage};
use serenity::prelude::{Context as SerenityContext, EventHandler};
use shared::cache::CacheInterface;
use shared::config::Config;
use shared::database::prelude::{
	GuildData, GuildSubscription, Message as DatabaseMessage, ServerUserRelation, UserData,
	UserSubscription, Vocal as DatabaseVocal,
};
use songbird::Songbird;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, instrument, trace, warn};
pub struct BotData {
	pub config: Arc<Config>,
	pub bot_info: Arc<RwLock<Option<CurrentApplicationInfo>>>,
	pub anilist_cache: Arc<RwLock<CacheInterface>>,
	pub vndb_cache: Arc<RwLock<CacheInterface>>,
	pub already_launched: RwLock<bool>,
	pub apps: Arc<RwLock<HashMap<String, u128>>>,
	pub user_blacklist: Arc<RwLock<Vec<String>>>,
	pub db_connection: Arc<DatabaseConnection>,
	pub manager: Arc<Songbird>,
	pub http_client: Arc<Client>,
	pub shard_manager: Arc<
		RwLock<
			Option<Arc<DashMap<ShardId, (ShardRunnerInfo, UnboundedSender<ShardRunnerMessage>)>>>,
		>,
	>,
	pub lavalink: Arc<RwLock<Option<LavalinkClient>>>,
	pub shutdown_signal: Arc<tokio::sync::broadcast::Sender<()>>,
	pub vocal_session: Arc<RwLock<HashMap<(String, String), DateTime<Utc>>>>,
	pub user_color_update_count: Arc<AtomicUsize>,
}
use crate::helper::load_items::load_items_from_json;
use crate::launch_task::thread_management_launcher;
use crate::music_events;
use anyhow::Result;
use dashmap::DashMap;
use futures::channel::mpsc::UnboundedSender;
use lavalink_rs::client::LavalinkClient;
use lavalink_rs::model::events;
use lavalink_rs::node::NodeBuilder;
use lavalink_rs::prelude::NodeDistributionStrategy;

pub struct Handler;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]

pub struct UserUsage {
	pub user_name: String,
	pub usage: u128,
	pub hourly_usage: HashMap<String, u128>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]

pub struct UserInfo {
	pub user_info: HashMap<String, UserUsage>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]

pub struct RootUsage {
	pub command_list: HashMap<String, UserInfo>,
}

impl RootUsage {
	#[allow(dead_code)]
	pub fn get_total_command_use(&self) -> String {
		let mut total = BigUint::ZERO;

		let command_usage = self.clone();

		for (_, user_info) in command_usage.command_list.iter() {
			for (_, user_usage) in user_info.user_info.iter() {
				total += user_usage.usage;
			}
		}

		total.to_string()
	}
}

use sea_orm::{ActiveModelTrait, ColumnTrait, PaginatorTrait, QueryFilter};

impl BotData {
	pub async fn get_hourly_usage(&self, command_name: String, user_id: String) -> u128 {
		// Query database for command usage in the current hour
		let conn = self.db_connection.clone();
		let now = chrono::Utc::now();
		let hour_start = now
			.date_naive()
			.and_hms_opt(now.hour(), 0, 0)
			.unwrap()
			.and_utc();
		let hour_end = hour_start + chrono::Duration::hours(1);

		match shared::database::command_usage::Entity::find()
			.filter(shared::database::command_usage::Column::Command.eq(command_name))
			.filter(shared::database::command_usage::Column::User.eq(user_id))
			.filter(shared::database::command_usage::Column::UseTime.gte(hour_start))
			.filter(shared::database::command_usage::Column::UseTime.lt(hour_end))
			.count(&*conn)
			.await
		{
			Ok(count) => count as u128,
			Err(e) => {
				error!("DB error reading command usage: {}", e);
				0u128
			},
		}
	}

	// Insert command usage record to DB
	pub async fn increment_command_use_per_command(
		&self, command_name: String, user_id: String, _user_name: String,
	) {
		let conn = self.db_connection.clone();
		let now = chrono::Utc::now().naive_utc();

		let am = shared::database::command_usage::ActiveModel {
			command: Set(command_name.clone()),
			user: Set(user_id.clone()),
			use_time: Set(now),
		};

		if let Err(e) = am.insert(&*conn).await {
			error!("Failed to insert command usage in DB: {}", e);
		}
	}
}

#[async_trait]

impl EventHandler for Handler {
	async fn dispatch(&self, ctx: &SerenityContext, event: &FullEvent) {
		match event {
			FullEvent::GuildCreate { guild, is_new } => {
				self.guild_create(ctx.clone(), guild.clone(), *is_new).await;
			},
			FullEvent::GuildMemberAddition { new_member } => {
				self.guild_member_addition(ctx.clone(), new_member.clone())
					.await;
			},
			FullEvent::GuildMembersChunk { chunk } => {
				self.guild_members_chunk(ctx.clone(), chunk.clone()).await;
			},
			FullEvent::PresenceUpdate { old_data, new_data } => {
				self.presence_update(ctx.clone(), old_data.clone(), new_data.clone())
					.await;
			},
			FullEvent::Ready { data_about_bot } => {
				self.ready(ctx.clone(), data_about_bot.clone()).await;
			},

			FullEvent::InteractionCreate { interaction } => {
				self.interaction_create(ctx.clone(), interaction.clone())
					.await;
			},
			FullEvent::EntitlementCreate { entitlement } => {
				self.entitlement_create(ctx.clone(), entitlement.clone())
					.await;
			},
			FullEvent::EntitlementUpdate { entitlement } => {
				self.entitlement_update(ctx.clone(), entitlement.clone())
					.await;
			},
			FullEvent::EntitlementDelete { entitlement } => {
				self.entitlement_delete(ctx.clone(), entitlement.clone())
					.await;
			},
			FullEvent::Message { new_message } => {
				self.new_message(ctx.clone(), new_message.clone()).await;
			},
			FullEvent::VoiceStateUpdate { old, new } => {
				self.voice_state_update(ctx.clone(), old.clone(), new.clone())
					.await;
			},
			_ => {
				trace!("this event is not handled nothing to worry {:?}", event)
			},
		}
	}
}
impl Handler {
	#[instrument(skip(self, ctx, old, new))]
	async fn voice_state_update(
		&self, ctx: SerenityContext, old: Option<VoiceState>, new: VoiceState,
	) {
		let user_id = new.user_id;
		trace!(user_id = %user_id, "Voice state update received for user");

		if let Some(old) = old {
			trace!(
				user_id = %user_id,
				"Old voice state is present for user"
			);

			match (new.channel_id, old.channel_id) {
				(None, None) => {
					trace!(user_id = %user_id, "User is not in a voice channel");
				},
				(Some(_), Some(_)) => {
					trace!(user_id = %user_id, "User switched voice channels");
				},
				(Some(new_channel_id), None) => {
					info!(user_id = %user_id, channel_id = %new_channel_id, "User joined a voice channel");
					let bot_data = ctx.data::<BotData>().clone();
					let key = (user_id.to_string(), new_channel_id.to_string());

					let mut rw_guard = bot_data.vocal_session.write().await;
					let sessions = rw_guard.get(&key).cloned();
					match sessions {
						Some(_) => {
							trace!(user_id = %user_id, channel_id = %new_channel_id, "Session already exists for user in channel");
						},
						None => {
							rw_guard.insert(key.clone(), Utc::now());
							info!(user_id = %user_id, channel_id = %new_channel_id, "Started new vocal session for user in channel");
						},
					}
					drop(rw_guard);
				},
				(None, Some(old_channel_id)) => {
					info!(user_id = %user_id, channel_id = %old_channel_id, "User left a voice channel");
					let bot_data = ctx.data::<BotData>().clone();
					let key = (user_id.to_string(), old_channel_id.to_string());

					let read = bot_data.vocal_session.read().await;
					let sessions = read.get(&key).cloned();
					drop(read);

					match sessions {
						Some(start_time) => {
							let mut write = bot_data.vocal_session.write().await;
							write.remove(&key);
							drop(write);

							let session_id = old.session_id.to_string();

							let start = start_time.naive_utc();
							let end = Utc::now().naive_utc();
							let duration = end.signed_duration_since(start).as_seconds_f64();
							let db_connection = bot_data.db_connection.clone();

							let id = format!("{}-{}-{}", user_id, old_channel_id, session_id);

							info!(
								user_id = %user_id, channel_id = %old_channel_id, session_id = %session_id,
								start = %start, end = %end, duration = duration,
								"Saving vocal session to database"
							);

							match DatabaseVocal::insert(shared::database::vocal::ActiveModel {
								id: Set(id),
								user_id: Set(user_id.to_string()),
								start: Set(start),
								end: Set(end),
								duration: Set(duration as i32),
								channel_id: Set(old_channel_id.to_string()),
							})
							.exec(&*db_connection.clone())
							.await
							{
								Ok(_) => {
									info!(user_id = %user_id, channel_id = %old_channel_id, "Vocal session saved to database");
								},
								Err(e) => {
									warn!(user_id = %user_id, channel_id = %old_channel_id, error = %e, "Failed to insert vocal session into database");
								},
							};
						},
						None => {
							trace!(user_id = %user_id, channel_id = %old_channel_id, "No session found to end for user in channel");
						},
					}
				},
			}
		} else {
			trace!(user_id = %user_id, "Old voice state is not present for user");
			if let Some(new_channel_id) = new.channel_id {
				info!(user_id = %user_id, channel_id = %new_channel_id, "User joined a voice channel");
				let bot_data = ctx.data::<BotData>().clone();
				let key = (user_id.to_string(), new_channel_id.to_string());

				let mut rw_guard = bot_data.vocal_session.write().await;
				let sessions = rw_guard.get(&key).cloned();
				match sessions {
					Some(_) => {
						trace!(user_id = %user_id, channel_id = %new_channel_id, "Session already exists for user in channel");
					},
					None => {
						rw_guard.insert(key.clone(), Utc::now());
						info!(user_id = %user_id, channel_id = %new_channel_id, "Started new vocal session for user in channel");
					},
				}
				drop(rw_guard);
			}
		}
	}

	async fn new_message(&self, ctx: SerenityContext, message: Message) {
		let bot_data = ctx.data::<BotData>().clone();
		let user_blacklist = bot_data.user_blacklist.clone();
		let read_guard = user_blacklist.read().await;
		let user_id = message.author.id;

		if read_guard.contains(&user_id.to_string()) {
			return;
		}
		trace!(
			message_id = %message.id,
			user_id = %user_id,
			"New message received"
		);

		let db_connection = bot_data.db_connection.clone();
		let message_id = message.id.to_string();
		let data = message.content.to_string();
		let length = data.len();
		let channel_id = message.channel_id.to_string();

		let active_message = shared::database::message::ActiveModel {
			id: Set(message_id),
			user_id: Set(user_id.to_string()),
			data: Set(data),
			chat_length: Set(length as i32),
			channel_id: Set(channel_id),
		};

		if let Err(e) = DatabaseMessage::insert(active_message)
			.exec(&*db_connection)
			.await
		{
			warn!(
				message_id = %message.id,
				user_id = %user_id,
				error = %e,
				"Failed to insert message into database"
			);
		}
	}

	async fn guild_create(&self, ctx: SerenityContext, guild: Guild, is_new: Option<bool>) {
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
			server_image_management(&ctx, image_config, db_connection.clone()).await;
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

	async fn guild_member_addition(&self, ctx: SerenityContext, member: Member) {
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
			bot_data.config.db.clone(),
		)
		.await;

		let count = bot_data
			.user_color_update_count
			.fetch_add(1, Ordering::SeqCst);
		if count >= 100 {
			bot_data.user_color_update_count.store(0, Ordering::SeqCst);
			let ctx_clone = ctx.clone();
			let image_config_clone = image_config.clone();
			let db_clone = bot_data.db_connection.clone();
			tokio::spawn(async move {
				server_image_management(&ctx_clone, image_config_clone, db_clone).await;
			});
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

	async fn guild_members_chunk(&self, ctx: SerenityContext, chunk: GuildMembersChunkEvent) {
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

	async fn presence_update(
		&self, ctx: SerenityContext, _old_data: Option<Presence>, new_data: Presence,
	) {
		let bot_data = ctx.data::<BotData>().clone();
		let user_blacklist_server_image = bot_data.user_blacklist.clone();
		let user_id = new_data.user.id;
		trace!(user_id = %user_id, "Presence update received");

		let user = new_data.user.to_user();

		if let Some(user) = user {
			get_specific_user_color(
				user_blacklist_server_image,
				user.clone(),
				bot_data.config.db.clone(),
			)
			.await;

			if let Err(e) = add_user_data_to_db(user, bot_data.db_connection.clone()).await {
				warn!(
					user_id = %user_id,
					error = ?e,
					"Failed to insert user data from presence update into database"
				);
			}
		}
	}

	async fn ready(&self, ctx: SerenityContext, ready: Ready) {
		let bot_data = ctx.data::<BotData>().clone();
		if bot_data.lavalink.read().await.is_none() {
			let events = events::Events {
				raw: Some(music_events::raw_event),
				ready: Some(music_events::ready_event),
				track_start: Some(music_events::track_start),
				..Default::default()
			};

			let user_id = lavalink_rs::model::UserId::from(ctx.cache.current_user().id.get());

			let music_config = bot_data
				.config
				.music
				.as_ref()
				.expect("Music configuration is required");

			let node_local = NodeBuilder {
				hostname: music_config.lavalink_hostname.clone(),
				is_ssl: music_config.https,
				events: events::Events::default(),
				password: music_config.lavalink_password.clone(),
				user_id,
				session_id: None,
			};

			let client = LavalinkClient::new(
				events,
				vec![node_local],
				NodeDistributionStrategy::round_robin(),
			)
			.await;
			*bot_data.lavalink.write().await = Some(client);
		}

		for guild in ctx.cache.guilds() {
			let partial_guild = match guild.to_partial_guild(&ctx.http).await {
				Ok(guild) => guild,
				Err(e) => {
					warn!(guild_id = %guild, error = %e, "Failed to get partial guild");
					continue;
				},
			};
			ctx.chunk_guild(partial_guild.id, None, true, ChunkGuildFilter::None, None);
			trace!(
				guild_id = %partial_guild.id,
				"Chunking guild"
			);
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
			command_registration(&ctx.http, bot_data.config.bot.remove_old_commands).await;

			let ctx_clone = ctx.clone();
			let image_config_clone = bot_data.config.image.clone();
			let db_clone = bot_data.db_connection.clone();
			tokio::spawn(async move {
				server_image_management(&ctx_clone, image_config_clone, db_clone).await;
			});
		}

		let db = bot_data.db_connection.clone();
		if let Err(e) = load_items_from_json(&db).await {
			warn!(error = %e, "Failed to load items from JSON");
		}
	}

	async fn interaction_create(&self, ctx: SerenityContext, interaction: Interaction) {
		let mut user = None;
		let bot_data = ctx.data::<BotData>().clone();
		trace!("Interaction received: {:?}", interaction.kind());

		match interaction.clone() {
			Interaction::Command(command_interaction) => {
				let mut message = String::from("");
				match command_interaction.data.kind {
					CommandType::ChatInput => {
						if let Err(e) = dispatch_command(&ctx, &command_interaction).await {
							message = e.to_string();
						} else {
							return;
						}
					},
					CommandType::User => {
						if let Err(e) = dispatch_user_command(&ctx, &command_interaction).await {
							message = e.to_string();
						} else {
							return;
						}
					},
					CommandType::Message => trace!("{:?}", command_interaction),
					_ => {},
				}
				error_dispatch::command_dispatching(message, &command_interaction, &ctx).await;
				user = Some(command_interaction.user.clone());
			},
			Interaction::Autocomplete(autocomplete_interaction) => {
				user = Some(autocomplete_interaction.user.clone());
				autocomplete_dispatching(ctx, autocomplete_interaction).await;
			},
			Interaction::Component(component_interaction) => {
				user = Some(component_interaction.user.clone());
				let db_connection = bot_data.db_connection.clone();
				if let Err(e) =
					components_dispatching(ctx, component_interaction, db_connection).await
				{
					warn!(error = ?e, "Failed to dispatch component interaction");
				}
			},
			_ => {},
		}

		if let Some(user) = user {
			if let Err(e) = add_user_data_to_db(user.clone(), bot_data.db_connection.clone()).await
			{
				warn!(
					user_id = %user.id,
					error = ?e,
					"Failed to insert user data from interaction into database"
				);
			}
		}
	}

	async fn entitlement_create(&self, ctx: SerenityContext, entitlement: Entitlement) {
		let bot_data = ctx.data::<BotData>().clone();
		let connection = bot_data.db_connection.clone();
		info!(
			entitlement_id = %entitlement.id,
			"New entitlement created"
		);
		insert_subscription(entitlement, connection).await;
	}

	async fn entitlement_update(&self, ctx: SerenityContext, entitlement: Entitlement) {
		let bot_data = ctx.data::<BotData>().clone();
		let connection = bot_data.db_connection.clone();
		info!(
			entitlement_id = %entitlement.id,
			"Entitlement updated"
		);
		insert_subscription(entitlement, connection).await;
	}

	async fn entitlement_delete(&self, ctx: SerenityContext, entitlement: Entitlement) {
		let bot_data = ctx.data::<BotData>().clone();
		let connection = bot_data.db_connection.clone();
		info!(
			entitlement_id = %entitlement.id,
			"Entitlement deleted"
		);
		let (guild_id, user_id) = (entitlement.guild_id, entitlement.user_id);
		let sku_id = entitlement.sku_id.to_string();

		if let Some(guild_id) = guild_id {
			let guild_id = guild_id.to_string();
			if let Err(e) = GuildSubscription::delete_by_id((guild_id.clone(), sku_id.clone()))
				.exec(&*connection)
				.await
			{
				warn!(
					guild_id = %guild_id,
					sku_id = %sku_id,
					error = %e,
					"Failed to delete guild subscription"
				);
			}
		} else if let Some(user_id) = user_id {
			let user_id = user_id.to_string();
			if let Err(e) = UserSubscription::delete_by_id((user_id.clone(), sku_id.clone()))
				.exec(&*connection)
				.await
			{
				warn!(
					user_id = %user_id,
					sku_id = %sku_id,
					error = %e,
					"Failed to delete user subscription"
				);
			}
		}
	}
}

async fn insert_subscription(entitlement: Entitlement, connection: Arc<DatabaseConnection>) {
	info!(
		entitlement_id = %entitlement.id,
		"Inserting or updating subscription"
	);
	match (entitlement.guild_id, entitlement.user_id) {
		(Some(guild_id), None) => {
			insert_guild_subscription(entitlement, guild_id.to_string(), connection).await
		},
		(None, Some(user_id)) => {
			insert_user_subscription(entitlement, user_id.to_string(), connection).await
		},
		_ => {},
	}
}

async fn insert_guild_subscription(
	entitlement: Entitlement, guild_id: String, connection: Arc<DatabaseConnection>,
) {
	let model = shared::database::guild_subscription::ActiveModel {
		guild_id: Set(guild_id.clone()),
		entitlement_id: Set(entitlement.id.to_string()),
		sku_id: Set(entitlement.sku_id.to_string()),
		created_at: Set(entitlement.starts_at.unwrap_or_default().naive_utc()),
		updated_at: Default::default(),
		expired_at: Default::default(),
	};
	if let Err(e) = GuildSubscription::insert(model)
		.on_conflict(
			sea_orm::sea_query::OnConflict::columns([
				shared::database::guild_subscription::Column::GuildId,
				shared::database::guild_subscription::Column::SkuId,
			])
			.update_columns([
				shared::database::guild_subscription::Column::EntitlementId,
				shared::database::guild_subscription::Column::ExpiredAt,
				shared::database::guild_subscription::Column::UpdatedAt,
			])
			.to_owned(),
		)
		.exec(&*connection)
		.await
	{
		warn!(
			guild_id = %guild_id,
			sku_id = %entitlement.sku_id,
			error = %e,
			"Failed to insert or update guild subscription"
		);
	}
}

async fn insert_user_subscription(
	entitlement: Entitlement, user_id: String, connection: Arc<DatabaseConnection>,
) {
	let model = shared::database::user_subscription::ActiveModel {
		user_id: Set(user_id.clone()),
		entitlement_id: Set(entitlement.id.to_string()),
		sku_id: Set(entitlement.sku_id.to_string()),
		created_at: Set(entitlement.starts_at.unwrap_or_default().naive_utc()),
		updated_at: Default::default(),
		expired_at: Default::default(),
	};
	if let Err(e) = UserSubscription::insert(model)
		.on_conflict(
			sea_orm::sea_query::OnConflict::columns([
				shared::database::user_subscription::Column::UserId,
				shared::database::user_subscription::Column::SkuId,
			])
			.update_columns([
				shared::database::user_subscription::Column::EntitlementId,
				shared::database::user_subscription::Column::ExpiredAt,
				shared::database::user_subscription::Column::UpdatedAt,
			])
			.to_owned(),
		)
		.exec(&*connection)
		.await
	{
		warn!(
			user_id = %user_id,
			sku_id = %entitlement.sku_id,
			error = %e,
			"Failed to insert or update user subscription"
		);
	}
}

pub async fn add_user_data_to_db(user: User, connection: Arc<DatabaseConnection>) -> Result<()> {
	trace!(user_id = %user.id, "Adding user data to database");
	let model = shared::database::user_data::ActiveModel {
		user_id: Set(user.id.to_string()),
		username: Set(user.name.to_string()),
		added_at: Set(Utc::now().naive_utc()),
		is_bot: Set(user.bot()),
	};
	if let Err(e) = UserData::insert(model)
		.on_conflict(
			sea_orm::sea_query::OnConflict::column(shared::database::user_data::Column::UserId)
				.update_columns([
					shared::database::user_data::Column::Username,
					shared::database::user_data::Column::IsBot,
				])
				.to_owned(),
		)
		.exec(&*connection)
		.await
	{
		warn!(
			user_id = %user.id,
			error = %e,
			"Failed to insert or update user data in database"
		);
		Err(e.into())
	} else {
		Ok(())
	}
}
