use crate::event_handler::{BotData, Handler};
use chrono::Utc;
use sea_orm::ActiveValue::Set;
use sea_orm::EntityTrait;
use serenity::all::VoiceState;
use serenity::prelude::Context as SerenityContext;
use shared::database::prelude::Vocal as DatabaseVocal;
use tracing::{info, instrument, trace, warn};

impl Handler {
	#[instrument(skip(self, ctx, old, new))]
	pub(crate) async fn voice_state_update(
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
}
