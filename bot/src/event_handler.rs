use crate::autocomplete::autocomplete_dispatch::autocomplete_dispatching;
use crate::background_task::background_launcher::thread_management_launcher;
use crate::background_task::server_image::calculate_user_color::{
	color_management, get_specific_user_color,
};
use crate::background_task::server_image::generate_server_image::server_image_management;
use crate::command::command_dispatch::{check_if_module_is_on, dispatch_command};
use crate::command::user_command_dispatch::dispatch_user_command;
use crate::components::components_dispatch::components_dispatching;
use crate::config::Config;
use crate::constant::COMMAND_USE_PATH;
use crate::database::prelude::{
	GuildData, GuildSubscription, ServerUserRelation, UserData, UserSubscription,
};
use crate::error_management::error_dispatch;
use crate::register::registration_dispatcher::command_registration;
use chrono::Utc;
use moka::future::Cache;
use num_bigint::BigUint;
use reqwest::Client;
use sea_orm::ActiveValue::Set;
use sea_orm::{DatabaseConnection, EntityTrait};
use serde::{Deserialize, Serialize};
use serenity::all::FullEvent;
use serenity::all::{
	CommandType, CurrentApplicationInfo, Entitlement, Guild, GuildMembersChunkEvent, Interaction,
	Member, Presence, Ready, ShardId, User,
};
use serenity::async_trait;
use serenity::gateway::{ActivityData, ChunkGuildFilter, ShardRunnerInfo};
use serenity::prelude::{Context as SerenityContext, EventHandler};
use songbird::Songbird;
use std::collections::HashMap;
use std::ops::{Add, AddAssign};
use std::sync::{Arc, Mutex};
use tokio::sync::RwLock;
use tracing::{debug, error, info, trace};

pub struct BotData {
	pub number_of_command_use_per_command: Arc<RwLock<RootUsage>>,
	pub config: Arc<Config>,
	pub bot_info: Arc<RwLock<Option<CurrentApplicationInfo>>>,
	pub anilist_cache: Arc<RwLock<Cache<String, String>>>,
	pub vndb_cache: Arc<RwLock<Cache<String, String>>>,
	pub already_launched: RwLock<bool>,
	pub apps: Arc<RwLock<HashMap<String, u128>>>,
	pub user_blacklist_server_image: Arc<RwLock<Vec<String>>>,
	pub db_connection: Arc<DatabaseConnection>,
	pub manager: Arc<Songbird>,
	pub http_client: Client,
	pub shard_manager: Arc<RwLock<Option<Arc<HashMap<ShardId, Arc<Mutex<ShardRunnerInfo>>>>>>>,
	pub lavalink: Arc<RwLock<Option<LavalinkClient>>>,
}
use crate::music_events;
use anyhow::{Context, Result};
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
	pub fn new() -> Self {
		RootUsage {
			command_list: HashMap::new(),
		}
	}

	pub fn get_total_command_use(&self) -> String {
		let mut total = BigUint::ZERO;

		let command_usage = self.clone();

		for (_, user_info) in command_usage.command_list.iter() {
			for (_, user_usage) in user_info.user_info.iter() {
				total.add_assign(user_usage.usage)
			}
		}

		total.to_string()
	}
}

impl BotData {
	pub async fn get_hourly_usage(&self, command_name: String, user_id: String) -> u128 {
		let number_of_command_use_per_command = self.number_of_command_use_per_command.clone();

		let guard = number_of_command_use_per_command.read().await;

		let user_map = guard
			.command_list
			.get(&command_name)
			.cloned()
			.unwrap_or_default()
			.user_info
			.get(&user_id)
			.cloned()
			.unwrap_or_default();

		*user_map
			.hourly_usage
			.get(&chrono::Local::now().format("%H").to_string())
			.unwrap_or(&(0u128))
	}

	// thread safe way to increment the number of command use per command
	pub async fn increment_command_use_per_command(
		&self, command_name: String, user_id: String, user_name: String,
	) {
		let number_of_command_use_per_command = self.number_of_command_use_per_command.clone();

		let mut guard = number_of_command_use_per_command.write().await;

		let command_map = guard
			.command_list
			.entry(command_name)
			.or_insert_with(|| UserInfo {
				user_info: HashMap::new(),
			});

		let user_map = command_map
			.user_info
			.entry(user_id)
			.or_insert_with(|| UserUsage {
				user_name,
				usage: 0,
				hourly_usage: Default::default(),
			});

		user_map.usage = user_map.usage.add(1);

		// create a timestamp in format dd:mm:aaaa:hh
		let timestamp = chrono::Local::now().format("%d:%m:%Y:%H").to_string();

		// insert or update the hourly usage
		let hourly_usage = user_map.hourly_usage.entry(timestamp).or_insert(0);

		*hourly_usage += 1;

		// drop the guard
		drop(guard);

		// save the content as a json
		match serde_json::to_string(&*self.number_of_command_use_per_command.read().await) {
			Ok(content) => {
				// save the content to the file
				if let Err(e) = std::fs::write(COMMAND_USE_PATH, content) {
					error!("Failed to write to file: {}", e);
				}
			},
			Err(e) => error!("Error serializing data: {}", e),
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
			_ => {
				trace!("this event is not handled nothing to worry")
			},
		}
	}
}
impl Handler {
	async fn guild_create(&self, ctx: SerenityContext, guild: Guild, is_new: Option<bool>) {
		let bot_data = ctx.data::<BotData>().clone();

		let image_config = bot_data.config.image.clone();

		let user_blacklist_server_image = bot_data.user_blacklist_server_image.clone();

		if is_new.unwrap_or_default() {
			color_management(
				&ctx.cache.guilds(),
				&ctx,
				user_blacklist_server_image,
				bot_data.clone(),
			)
			.await;

			server_image_management(&ctx, image_config, bot_data.db_connection.clone()).await;

			debug!("Joined a new guild: {} at {}", guild.name, guild.joined_at);
		} else {
			debug!("Got info from guild: {} at {}", guild.name, guild.joined_at);
		}

		let connection = bot_data.db_connection.clone();

		match GuildData::insert(crate::database::guild_data::ActiveModel {
			guild_id: Set(guild.id.to_string()),
			guild_name: Set(guild.name.to_string()),
			updated_at: Set(guild.joined_at.naive_utc()),
		})
		.on_conflict(
			sea_orm::sea_query::OnConflict::column(crate::database::guild_data::Column::GuildId)
				.update_column(crate::database::guild_data::Column::GuildName)
				.update_column(crate::database::guild_data::Column::UpdatedAt)
				.to_owned(),
		)
		.exec(&*connection)
		.await
		{
			Ok(_) => {},
			Err(e) => error!("Failed to insert guild data. {}", e),
		};
	}

	async fn guild_member_addition(&self, ctx: SerenityContext, member: Member) {
		let bot_data = ctx.data::<BotData>().clone();

		let user_blacklist_server_image = bot_data.user_blacklist_server_image.clone();

		let guild_id = member.guild_id.to_string();

		let image_config = bot_data.config.image.clone();

		debug!(
			"Member {} joined guild {}",
			member.user.tag(),
			guild_id.clone()
		);

		let is_module_on =
			check_if_module_is_on(guild_id.clone(), "NEW_MEMBER", bot_data.config.db.clone())
				.await
				.unwrap_or_else(|e| {
					error!("Failed to get the module status. {}", e);

					false
				});

		color_management(
			&ctx.cache.guilds(),
			&ctx,
			user_blacklist_server_image,
			bot_data.clone(),
		)
		.await;

		if is_module_on {
			server_image_management(&ctx, image_config, bot_data.db_connection.clone()).await;
		}

		let user = match member.user.id.to_user(&ctx.http).await {
			Ok(user) => user,
			Err(e) => {
				error!("Failed to get user. {}", e);

				return;
			},
		};

		match add_user_data_to_db(user, bot_data.db_connection.clone()).await {
			Ok(_) => {},
			Err(e) => error!("Failed to insert user data. {}", e),
		};
	}

	async fn guild_members_chunk(&self, ctx: SerenityContext, chunk: GuildMembersChunkEvent) {
		let bot_data = ctx.data::<BotData>().clone();

		let members = chunk.members;

		if members.is_empty() {
			return;
		}

		let connection = bot_data.db_connection.clone();

		for member in members {
			let user = match member.user.id.to_user(&ctx.http).await {
				Ok(user) => user,
				Err(e) => {
					error!("Failed to get user. {}", e);

					return;
				},
			};

			match add_user_data_to_db(user.clone(), connection.clone()).await {
				Ok(_) => {},
				Err(e) => error!("Failed to insert user data. {}", e),
			};

			match ServerUserRelation::insert(crate::database::server_user_relation::ActiveModel {
				guild_id: Set(chunk.guild_id.to_string()),
				user_id: Set(user.id.to_string()),
			})
			.on_conflict(
				sea_orm::sea_query::OnConflict::columns([
					crate::database::server_user_relation::Column::GuildId,
					crate::database::server_user_relation::Column::UserId,
				])
				.do_nothing()
				.to_owned(),
			)
			.exec(&*connection.clone())
			.await
			{
				Ok(_) => {},
				Err(e) => error!("Failed to insert server user relation. {}", e),
			};
		}
	}

	async fn presence_update(
		&self, ctx: SerenityContext, _old_data: Option<Presence>, new_data: Presence,
	) {
		let bot_data = ctx.data::<BotData>().clone();

		let user_blacklist_server_image = bot_data.user_blacklist_server_image.clone();

		let user_id = new_data.user.id;

		debug!("Member {} updated presence", user_id);

		let user = match new_data.user.id.to_user(&ctx).await {
			Ok(user) => user,
			Err(e) => {
				error!("Failed to get the user. {}", e);

				return;
			},
		};

		get_specific_user_color(
			user_blacklist_server_image,
			user.clone(),
			bot_data.config.db.clone(),
		)
		.await;

		match add_user_data_to_db(user, bot_data.db_connection.clone()).await {
			Ok(_) => {},
			Err(e) => error!("Failed to insert user data. {}", e),
		};
	}

	async fn ready(&self, ctx: SerenityContext, ready: Ready) {
		let bot_data = ctx.data::<BotData>().clone();
		let guard = bot_data.lavalink.read().await;
		match *guard {
			None => {
				drop(guard);
				let events = events::Events {
					raw: Some(music_events::raw_event),
					ready: Some(music_events::ready_event),
					track_start: Some(music_events::track_start),
					..Default::default()
				};

				let user_id = lavalink_rs::model::UserId::from(ctx.cache.current_user().id.get());

				let node_local = NodeBuilder {
					hostname: bot_data.config.music.lavalink_hostname.clone(),
					is_ssl: bot_data.config.music.https,
					events: events::Events::default(),
					password: bot_data.config.music.lavalink_password.clone(),
					user_id,
					session_id: None,
				};

				let client = LavalinkClient::new(
					events,
					vec![node_local],
					NodeDistributionStrategy::round_robin(),
				)
				.await;
				let mut write_guard = bot_data.lavalink.write().await;

				*write_guard = Some(client);
				drop(write_guard)
			},
			_ => {
				drop(guard);
			},
		}

		for guild in ctx.cache.guilds() {
			// Retrieves partial guild information
			let partial_guild = match guild.to_partial_guild(&ctx.http).await {
				Ok(guild) => guild,
				Err(e) => {
					error!("Failed to get the guild. {}", e);

					continue;
				},
			};
			// Iterates over each guild the bot is in
			ctx.chunk_guild(partial_guild.id, None, true, ChunkGuildFilter::None, None);

			debug!(
				"guild name {} (guild id: {})",
				&partial_guild.name,
				&partial_guild.id.to_string()
			)
		}

		// Logs a message indicating that the shard is connected
		info!(
			"Shard {:?} of {} is connected!",
			ready.shard, ready.user.name
		);
		ctx.set_activity(Some(ActivityData::custom(
			bot_data.config.bot.bot_activity.clone(),
		)));
		let guard = bot_data.already_launched.read().await;

		if !(*guard) {
			drop(guard);

			let mut write_guard = bot_data.already_launched.write().await;

			*write_guard = true;

			tokio::spawn(thread_management_launcher(
				ctx.clone(),
				bot_data.clone(),
				bot_data.config.db.clone(),
			));

			drop(write_guard);

			let remove_old_command = bot_data.config.bot.remove_old_commands;

			command_registration(&ctx.http, remove_old_command).await;
		}
	}

	async fn interaction_create(&self, ctx: SerenityContext, interaction: Interaction) {
		let mut user = None;

		let bot_data = ctx.data::<BotData>().clone();

		if let Interaction::Command(command_interaction) = interaction.clone() {
			let mut message = String::from("");

			if command_interaction.data.kind == CommandType::ChatInput {
				match dispatch_command(&ctx, &command_interaction).await {
					Ok(()) => return,
					Err(e) => {
						message = e.to_string();
					},
				}
			} else if command_interaction.data.kind == CommandType::User {
				match dispatch_user_command(&ctx, &command_interaction).await {
					Ok(()) => return,
					Err(e) => {
						message = e.to_string();
					},
				}
			} else if command_interaction.data.kind == CommandType::Message {
				trace!("{:?}", command_interaction)
			}

			error_dispatch::command_dispatching(message, &command_interaction, &ctx).await;

			user = Some(command_interaction.user.clone());
		} else if let Interaction::Autocomplete(autocomplete_interaction) = interaction.clone() {
			// Dispatch the autocomplete interaction
			user = Some(autocomplete_interaction.user.clone());

			autocomplete_dispatching(ctx, autocomplete_interaction).await;
		} else if let Interaction::Component(component_interaction) = interaction.clone() {
			// Dispatch the component interaction
			user = Some(component_interaction.user.clone());

			if let Err(e) =
				components_dispatching(ctx, component_interaction, bot_data.config.db.clone()).await
			{
				// If an error occurs, log it
				error!("{:?}", e)
			}
		}

		if user.is_none() {
			return;
		}

		match add_user_data_to_db(user.unwrap(), bot_data.db_connection.clone()).await {
			Ok(_) => {},
			Err(e) => error!("Failed to insert user data. {}", e),
		};
	}

	async fn entitlement_create(&self, ctx: SerenityContext, entitlement: Entitlement) {
		let bot_data = ctx.data::<BotData>().clone();

		let connection = bot_data.db_connection.clone();

		insert_subscription(entitlement, connection).await;
	}

	async fn entitlement_update(&self, ctx: SerenityContext, entitlement: Entitlement) {
		let bot_data = ctx.data::<BotData>().clone();

		let connection = bot_data.db_connection.clone();

		insert_subscription(entitlement, connection).await;
	}

	async fn entitlement_delete(&self, ctx: SerenityContext, entitlement: Entitlement) {
		let bot_data = ctx.data::<BotData>().clone();

		let connection = bot_data.db_connection.clone();

		match (entitlement.guild_id, entitlement.user_id) {
			(Some(guild_id), None) => {
				let guild_id = guild_id.to_string();

				let sku_id = entitlement.sku_id.to_string();

				match GuildSubscription::delete_by_id((guild_id, sku_id))
					.exec(&*connection.clone())
					.await
				{
					Ok(_) => {},
					Err(e) => error!("Failed to delete guild subscription. {}", e),
				};
			},
			(None, Some(user_id)) => {
				let user_id = user_id.to_string();

				let sku_id = entitlement.sku_id.to_string();

				match UserSubscription::delete_by_id((user_id, sku_id))
					.exec(&*connection.clone())
					.await
				{
					Ok(_) => {},
					Err(e) => error!("Failed to delete user subscription. {}", e),
				};
			},
			_ => {},
		}
	}
}

async fn insert_subscription(entitlement: Entitlement, connection: Arc<DatabaseConnection>) {
	match (entitlement.guild_id, entitlement.user_id) {
		(Some(guild_id), None) => {
			let guild_id = guild_id.to_string();

			insert_guild_subscription(entitlement, guild_id, connection.clone()).await;
		},
		(None, Some(user_id)) => {
			let user_id = user_id.to_string();

			insert_user_subscription(entitlement, user_id, connection.clone()).await;
		},
		_ => {},
	}
}

async fn insert_guild_subscription(
	entitlement: Entitlement, guild_id: String, connection: Arc<DatabaseConnection>,
) {
	match GuildSubscription::insert(crate::database::guild_subscription::ActiveModel {
		guild_id: Set(guild_id),
		entitlement_id: Set(entitlement.id.to_string()),
		sku_id: Set(entitlement.sku_id.to_string()),
		created_at: Set(entitlement.starts_at.unwrap_or_default().naive_utc()),
		updated_at: Default::default(),
		expired_at: Default::default(),
	})
	.on_conflict(
		sea_orm::sea_query::OnConflict::columns([
			crate::database::guild_subscription::Column::GuildId,
			crate::database::guild_subscription::Column::SkuId,
		])
		.update_columns([
			crate::database::guild_subscription::Column::EntitlementId,
			crate::database::guild_subscription::Column::ExpiredAt,
			crate::database::guild_subscription::Column::UpdatedAt,
		])
		.to_owned(),
	)
	.exec(&*connection.clone())
	.await
	{
		Ok(_) => {},
		Err(e) => error!("Failed to insert guild subscription. {}", e),
	};
}

async fn insert_user_subscription(
	entitlement: Entitlement, user_id: String, connection: Arc<DatabaseConnection>,
) {
	match UserSubscription::insert(crate::database::user_subscription::ActiveModel {
		user_id: Set(user_id),
		entitlement_id: Set(entitlement.id.to_string()),
		sku_id: Set(entitlement.sku_id.to_string()),
		created_at: Set(entitlement.starts_at.unwrap_or_default().naive_utc()),
		updated_at: Default::default(),
		expired_at: Default::default(),
	})
	.on_conflict(
		sea_orm::sea_query::OnConflict::columns([
			crate::database::user_subscription::Column::UserId,
			crate::database::user_subscription::Column::SkuId,
		])
		.update_columns([
			crate::database::user_subscription::Column::EntitlementId,
			crate::database::user_subscription::Column::ExpiredAt,
			crate::database::user_subscription::Column::UpdatedAt,
		])
		.to_owned(),
	)
	.exec(&*connection.clone())
	.await
	{
		Ok(_) => {},
		Err(e) => error!("Failed to insert user subscription. {}", e),
	};
}

pub async fn add_user_data_to_db(user: User, connection: Arc<DatabaseConnection>) -> Result<()> {
	UserData::insert(crate::database::user_data::ActiveModel {
		user_id: Set(user.id.to_string()),
		username: Set(user.name.to_string()),
		added_at: Set(Utc::now().naive_utc()),
		is_bot: Set(user.bot()),
		banner: Set(user.banner_url().unwrap_or_default()),
	})
	.on_conflict(
		sea_orm::sea_query::OnConflict::column(crate::database::user_data::Column::UserId)
			.update_columns([
				crate::database::user_data::Column::Username,
				crate::database::user_data::Column::Banner,
				crate::database::user_data::Column::IsBot,
			])
			.to_owned(),
	)
	.exec(&*connection)
	.await
	.context("Failed to add user to db")?;

	Ok(())
}
