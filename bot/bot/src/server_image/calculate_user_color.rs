use std::sync::Arc;

use serenity::all::{Context as SerenityContext, GuildId, Member, User, UserId};
use serenity::nonmax::NonMaxU16;
use shared::queue::publisher::publish_task;
use shared::queue::tasks::ImageTask;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::event_handler::BotData;

pub fn change_to_x128_url(url: String) -> String {
	debug!("Changing URL size to 128x128: {}", url);
	let base_url = url.split('?').next().unwrap_or(&url);
	format!("{}?size=128&quality=lossless", base_url)
}

pub async fn get_member(ctx_clone: SerenityContext, guild: GuildId) -> Vec<Member> {
	if let Some(guild_cache) = guild.to_guild_cached(&ctx_clone.cache) {
		debug!("Using cached members for guild {}", guild);
		let members = guild_cache.members.clone();
		return members.into_iter().map(|m| m.into()).collect();
	}

	debug!("Cache miss for guild {}, fetching from API", guild);
	let mut i = 0;
	let mut members_temp_out: Vec<Member> = Vec::new();

	while members_temp_out.len() == (1000 * i) {
		let mut members_temp_in = if i == 0 {
			match guild
				.members(
					&ctx_clone.http,
					Some(NonMaxU16::new(1000).unwrap_or_default()),
					None,
				)
				.await
			{
				Ok(members) => members,
				Err(e) => {
					error!("{:?}", e);
					break;
				},
			}
		} else {
			let user: UserId = match members_temp_out.last() {
				Some(member) => member.user.id,
				None => break,
			};

			match guild
				.members(
					&ctx_clone.http,
					Some(NonMaxU16::new(1000).unwrap_or_default()),
					Some(user),
				)
				.await
			{
				Ok(members) => members,
				Err(e) => {
					error!("{:?}", e);
					break;
				},
			}
		};

		i += 1;
		members_temp_out.append(&mut members_temp_in);
	}

	members_temp_out
}

pub async fn enqueue_user_color(
	user_blacklist_server_image: Arc<RwLock<Vec<String>>>, user: User, bot_data: Arc<BotData>,
) {
	if user_blacklist_server_image
		.read()
		.await
		.contains(&user.id.to_string())
	{
		debug!(
			"Skipping user {} due to USER_BLACKLIST_SERVER_IMAGE",
			user.id
		);
		return;
	}

	let pfp_url = change_to_x128_url(user.face());

	let task = ImageTask::CalculateUserColor {
		user_id: user.id.to_string(),
		profile_picture_url: pfp_url,
	};

	let mut guard = match bot_data.get_redis_connection().await {
		Some(g) => g,
		None => {
			warn!(
				"Redis unavailable, cannot enqueue color calculation for user {}",
				user.id
			);
			return;
		},
	};
	if let Err(e) = publish_task(guard.as_mut().unwrap(), &task).await {
		error!("Failed to enqueue user color task for {}: {:#}", user.id, e);
	}
}

pub async fn color_management(
	guilds: &Vec<GuildId>, ctx_clone: &SerenityContext,
	user_blacklist_server_image: Arc<RwLock<Vec<String>>>, bot_data: Arc<BotData>,
) {
	info!("Starting color management for {} guilds", guilds.len());

	for guild in guilds.iter() {
		let guild_id = guild.to_string();
		debug!(guild_id);

		let members = get_member(ctx_clone.clone(), *guild).await;
		debug!("{}: {}", guild_id, members.len());

		for member in members {
			enqueue_user_color(
				user_blacklist_server_image.clone(),
				member.user,
				bot_data.clone(),
			)
			.await;
		}
	}

	info!("Completed color management for all guilds");
}
