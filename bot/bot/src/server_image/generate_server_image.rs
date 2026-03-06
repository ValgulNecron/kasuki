use std::sync::Arc;

use anyhow::{anyhow, Result};
use sea_orm::DatabaseConnection;
use serenity::all::{Context as SerenityContext, GuildId, Member};
use shared::config::ImageConfig;
use shared::queue::tasks::{ImageTask, MemberColorData};
use tracing::{info, warn};

use crate::event_handler::BotData;
use crate::server_image::calculate_user_color::get_member;

fn get_guild_icon_url(guild_icon_url: Option<String>) -> String {
	guild_icon_url
		.unwrap_or_else(|| String::from("https://cdn.discordapp.com/icons/1117152661620408531/541e10cc07361e99b7b1012861cd518a.webp?size=128&quality=lossless"))
}

pub async fn enqueue_local_server_image(
	ctx: &SerenityContext, guild_id: GuildId, _image_config: &ImageConfig,
	_connection: Arc<DatabaseConnection>,
) -> Result<()> {
	let bot_data = ctx.data::<BotData>().clone();

	let members: Vec<Member> = get_member(ctx.clone(), guild_id).await;

	let guild = guild_id
		.to_partial_guild(&ctx.http)
		.await
		.map_err(|e| anyhow!("Failed to get partial guild {}: {:?}", guild_id, e))?;

	let guild_icon_url = get_guild_icon_url(guild.icon_url());

	let user_blacklist = bot_data.user_blacklist.read().await.clone();

	let member_data: Vec<MemberColorData> = members
		.iter()
		.map(|member| MemberColorData {
			user_id: member.user.id.to_string(),
			profile_picture_url: member.user.face(),
		})
		.collect();

	let task = ImageTask::GenerateServerImage {
		guild_id: guild_id.to_string(),
		guild_name: guild.name.to_string(),
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

	let guild = guild_id
		.to_partial_guild(&ctx.http)
		.await
		.map_err(|e| anyhow!("Failed to get partial guild {}: {:?}", guild_id, e))?;

	let guild_icon_url = get_guild_icon_url(guild.icon_url());

	let user_blacklist = bot_data.user_blacklist.read().await.clone();

	let task = ImageTask::GenerateServerImage {
		guild_id: guild_id.to_string(),
		guild_name: guild.name.to_string(),
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
	ctx: &SerenityContext, image_config: ImageConfig, connection: Arc<DatabaseConnection>,
) {
	for guild in ctx.cache.guilds() {
		if let Err(e) =
			enqueue_local_server_image(ctx, guild, &image_config, connection.clone()).await
		{
			warn!(
				"Failed to enqueue local server image for guild {}. {:?}",
				guild, e
			);
		} else {
			info!("Enqueued local server image for guild {}", guild);
		}

		if let Err(e) =
			enqueue_global_server_image(ctx, guild, &image_config, connection.clone()).await
		{
			warn!(
				"Failed to enqueue global server image for guild {}. {:?}",
				guild, e
			);
		} else {
			info!("Enqueued global server image for guild {}", guild);
		}
	}
}
