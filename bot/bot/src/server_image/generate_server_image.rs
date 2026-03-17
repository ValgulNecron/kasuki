use std::sync::Arc;

use anyhow::{anyhow, Result};
use sea_orm::DatabaseConnection;
use serenity::all::{Context as SerenityContext, GuildId, User};
use shared::config::ImageConfig;
use shared::queue::tasks::{ImageTask, MemberColorData};
use tracing::{info, warn};

use crate::event_handler::BotData;
use crate::server_image::calculate_user_color::{enqueue_user_color, get_member};

fn get_guild_icon_url(guild_icon_url: Option<String>) -> String {
	guild_icon_url
		.unwrap_or_else(|| String::from("https://cdn.discordapp.com/icons/1117152661620408531/541e10cc07361e99b7b1012861cd518a.webp?size=128&quality=lossless"))
}

async fn get_guild_info(
	ctx: &SerenityContext, guild_id: GuildId,
) -> Result<(String, String)> {
	if let Some(guild_cache) = guild_id.to_guild_cached(&ctx.cache) {
		let name = guild_cache.name.to_string();
		let icon_url = get_guild_icon_url(guild_cache.icon_url());
		return Ok((name, icon_url));
	}

	let guild = guild_id
		.to_partial_guild(&ctx.http)
		.await
		.map_err(|e| anyhow!("Failed to get partial guild {}: {:?}", guild_id, e))?;

	Ok((guild.name.to_string(), get_guild_icon_url(guild.icon_url())))
}

pub async fn enqueue_local_server_image(
	ctx: &SerenityContext, guild_id: GuildId, _image_config: &ImageConfig,
	_connection: Arc<DatabaseConnection>,
) -> Result<()> {
	let bot_data = ctx.data::<BotData>().clone();

	let users: Vec<User> = get_member(ctx, guild_id).await;
	let (guild_name, guild_icon_url) = get_guild_info(ctx, guild_id).await?;

	let user_blacklist = bot_data.user_blacklist.read().await.clone();

	let member_data: Vec<MemberColorData> = users
		.iter()
		.map(|user| MemberColorData {
			user_id: user.id.to_string(),
			profile_picture_url: user.face(),
		})
		.collect();

	let task = ImageTask::GenerateServerImage {
		guild_id: guild_id.to_string(),
		guild_name,
		guild_icon_url,
		image_type: String::from("local"),
		members: member_data,
		blacklist: user_blacklist,
	};

	bot_data
		.server_image_task_tx
		.send(task)
		.map_err(|_| anyhow!("Server image queue publisher task stopped"))?;

	Ok(())
}

pub async fn enqueue_global_server_image(
	ctx: &SerenityContext, guild_id: GuildId, _image_config: &ImageConfig,
	_connection: Arc<DatabaseConnection>,
) -> Result<()> {
	let bot_data = ctx.data::<BotData>().clone();

	let (guild_name, guild_icon_url) = get_guild_info(ctx, guild_id).await?;

	let user_blacklist = bot_data.user_blacklist.read().await.clone();

	let task = ImageTask::GenerateServerImage {
		guild_id: guild_id.to_string(),
		guild_name,
		guild_icon_url,
		image_type: String::from("global"),
		members: Vec::new(),
		blacklist: user_blacklist,
	};

	bot_data
		.server_image_task_tx
		.send(task)
		.map_err(|_| anyhow!("Server image queue publisher task stopped"))?;

	Ok(())
}

pub async fn server_image_management(
	ctx: &SerenityContext, _image_config: ImageConfig, _connection: Arc<DatabaseConnection>,
) {
	let bot_data = ctx.data::<BotData>().clone();
	let user_blacklist = bot_data.user_blacklist.clone();
	let guilds = ctx.cache.guilds();

	info!(
		"Starting server image management for {} guilds",
		guilds.len()
	);

	for guild in &guilds {
		let users = get_member(ctx, *guild).await;

		for user in &users {
			enqueue_user_color(user_blacklist.clone(), user.clone(), bot_data.clone()).await;
		}

		let (guild_name, guild_icon_url) = match get_guild_info(ctx, *guild).await {
			Ok(info) => info,
			Err(e) => {
				warn!("Failed to get guild info for {}. {:?}", guild, e);
				continue;
			},
		};

		let user_blacklist_snapshot = bot_data.user_blacklist.read().await.clone();

		let member_data: Vec<MemberColorData> = users
			.iter()
			.map(|user| MemberColorData {
				user_id: user.id.to_string(),
				profile_picture_url: user.face(),
			})
			.collect();

		let local_task = ImageTask::GenerateServerImage {
			guild_id: guild.to_string(),
			guild_name: guild_name.clone(),
			guild_icon_url: guild_icon_url.clone(),
			image_type: String::from("local"),
			members: member_data,
			blacklist: user_blacklist_snapshot.clone(),
		};

		if let Err(_) = bot_data.server_image_task_tx.send(local_task) {
			warn!("Server image queue publisher stopped");
			return;
		}

		let global_task = ImageTask::GenerateServerImage {
			guild_id: guild.to_string(),
			guild_name,
			guild_icon_url,
			image_type: String::from("global"),
			members: Vec::new(),
			blacklist: user_blacklist_snapshot,
		};

		if let Err(_) = bot_data.server_image_task_tx.send(global_task) {
			warn!("Server image queue publisher stopped");
			return;
		}

		info!("Enqueued server images for guild {}", guild);

		tokio::time::sleep(std::time::Duration::from_millis(100)).await;
	}

	info!(
		"Server image management complete for {} guilds",
		guilds.len()
	);
}
