use std::collections::HashSet;
use std::sync::Arc;

use serenity::all::{Context as SerenityContext, GuildId, User, UserId};
use serenity::nonmax::NonMaxU16;
use shared::queue::tasks::ImageTask;
use tokio::sync::RwLock;
use tracing::{debug, error};

use crate::event_handler::BotData;

pub async fn get_member(ctx_clone: &SerenityContext, guild: GuildId) -> Vec<User> {
	if let Some(guild_cache) = guild.to_guild_cached(&ctx_clone.cache) {
		debug!("Using cached members for guild {}", guild);
		return guild_cache.members.iter().map(|m| m.user.clone()).collect();
	}

	debug!("Cache miss for guild {}, fetching from API", guild);
	let mut i = 0;
	let mut members_temp_out: Vec<User> = Vec::new();

	while members_temp_out.len() == (1000 * i) {
		let members_temp_in = if i == 0 {
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
				Some(u) => u.id,
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
		members_temp_out.extend(members_temp_in.into_iter().map(|m| m.user));
	}

	members_temp_out
}

pub async fn enqueue_user_color(
	user_blacklist_server_image: Arc<RwLock<HashSet<String>>>, user: User, bot_data: Arc<BotData>,
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

	let task = ImageTask::CalculateUserColor {
		user_id: user.id.to_string(),
		profile_picture_url: user.face(),
	};

	if let Err(_) = bot_data.user_color_task_tx.send(task) {
		error!("User color queue publisher stopped, dropping task for user {}", user.id);
	}
}
