use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{anyhow, Result};
use sea_orm::{DatabaseConnection, EntityTrait};
use serenity::all::{Context as SerenityContext, GuildId, Member};
use shared::config::ImageConfig;
use shared::database::prelude::UserColor;
use shared::database::user_color::Model;
use shared::queue::publisher::publish_task;
use shared::queue::tasks::{ImageSaveConfig, ImageTask, MemberColorData};
use tracing::{info, warn};

use crate::event_handler::BotData;
use crate::server_image::calculate_user_color::{change_to_x128_url, get_member};

fn build_image_save_config(image_config: &ImageConfig) -> ImageSaveConfig {
	ImageSaveConfig {
		save_type: image_config.save_image.clone(),
		save_server: image_config.save_server.clone().unwrap_or_default(),
		token: image_config.token.clone().unwrap_or_default(),
	}
}

fn get_guild_icon_url(guild_icon_url: Option<String>) -> String {
	change_to_x128_url(
		guild_icon_url
			.unwrap_or_else(|| String::from("https://cdn.discordapp.com/icons/1117152661620408531/541e10cc07361e99b7b1012861cd518a.webp?size=128&quality=lossless")),
	)
}

pub async fn enqueue_local_server_image(
	ctx: &SerenityContext, guild_id: GuildId, image_config: &ImageConfig,
	connection: Arc<DatabaseConnection>,
) -> Result<()> {
	let bot_data = ctx.data::<BotData>().clone();

	let members: Vec<Member> = get_member(ctx.clone(), guild_id).await;

	let guild = guild_id
		.to_partial_guild(&ctx.http)
		.await
		.map_err(|e| anyhow!("Failed to get partial guild {}: {:?}", guild_id, e))?;

	let guild_icon_url = get_guild_icon_url(guild.icon_url());

	let user_blacklist = bot_data.user_blacklist.read().await.clone();

	let all_colors = UserColor::find().all(&*connection).await?;
	let color_map: HashMap<String, Model> = all_colors
		.into_iter()
		.map(|c| (c.user_id.clone(), c))
		.collect();

	let mut member_data: Vec<MemberColorData> = Vec::with_capacity(members.len());
	for member in &members {
		let user_id = member.user.id.to_string();
		let pfp_url = change_to_x128_url(member.user.face());

		let (cached_color, cached_image) = match color_map.get(&user_id) {
			Some(record) if record.profile_picture_url == pfp_url => {
				(Some(record.color.clone()), Some(record.images.clone()))
			},
			_ => (None, None),
		};

		member_data.push(MemberColorData {
			user_id,
			profile_picture_url: pfp_url,
			cached_color,
			cached_image,
		});
	}

	let task = ImageTask::GenerateServerImage {
		guild_id: guild_id.to_string(),
		guild_name: guild.name.to_string(),
		guild_icon_url,
		image_type: String::from("local"),
		members: member_data,
		blacklist: user_blacklist,
		image_save_config: build_image_save_config(image_config),
	};

	let mut guard = bot_data
		.get_redis_connection()
		.await
		.ok_or_else(|| anyhow!("Redis unavailable, cannot enqueue local server image for guild {}", guild_id))?;
	publish_task(guard.as_mut().unwrap(), &task).await?;

	Ok(())
}

pub async fn enqueue_global_server_image(
	ctx: &SerenityContext, guild_id: GuildId, image_config: &ImageConfig,
	connection: Arc<DatabaseConnection>,
) -> Result<()> {
	let bot_data = ctx.data::<BotData>().clone();

	let guild = guild_id
		.to_partial_guild(&ctx.http)
		.await
		.map_err(|e| anyhow!("Failed to get partial guild {}: {:?}", guild_id, e))?;

	let guild_icon_url = get_guild_icon_url(guild.icon_url());

	let user_blacklist = bot_data.user_blacklist.read().await.clone();

	let all_colors = UserColor::find().all(&*connection).await?;

	let member_data: Vec<MemberColorData> = all_colors
		.into_iter()
		.map(|uc| MemberColorData {
			user_id: uc.user_id,
			profile_picture_url: uc.profile_picture_url,
			cached_color: Some(uc.color),
			cached_image: Some(uc.images),
		})
		.collect();

	let task = ImageTask::GenerateServerImage {
		guild_id: guild_id.to_string(),
		guild_name: guild.name.to_string(),
		guild_icon_url,
		image_type: String::from("global"),
		members: member_data,
		blacklist: user_blacklist,
		image_save_config: build_image_save_config(image_config),
	};

	let mut guard = bot_data
		.get_redis_connection()
		.await
		.ok_or_else(|| anyhow!("Redis unavailable, cannot enqueue global server image for guild {}", guild_id))?;
	publish_task(guard.as_mut().unwrap(), &task).await?;

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
