use std::error::Error;
use std::sync::Arc;
use serenity::all::{CommandInteraction, Context};
use songbird::CoreEvent;
use tracing::{error, trace};
use crate::audio::receiver::Receiver;
use crate::config::Config;
use crate::helper::error_management::error_enum::{FollowupError, ResponseError};

pub async fn run(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    config: Arc<Config>,
) -> Result<(), Box<dyn Error>> {
    let guild_id = command_interaction
        .guild_id
        .ok_or(ResponseError::Option(String::from("No guild id")))?;

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();
    let bind = manager.get(guild_id);
    trace!(?bind);
    let cache = ctx.cache.clone();

    if manager.get(guild_id).is_none() {
        let channel_id;
        {
            let guild = match guild_id.to_guild_cached(&cache) {
                Some(guild) => guild,
                None => {
                    error!("Failed to get the guild.");
                    return Err(Box::new(ResponseError::Option(
                        "Failed to get the guild.".to_string(),
                    )));
                }
            };
            channel_id = guild
                .voice_states
                .get(&command_interaction.user.id)
                .and_then(|voice_state| voice_state.channel_id);
        }
        trace!(?channel_id);
        let connect_to = match channel_id {
            Some(channel) => channel,
            None => {
                return Err(Box::new(FollowupError::Option(String::from(
                    "Not connected to a voice channel",
                ))))
            }
        };

        let manager = songbird::get(ctx)
            .await
            .expect("Songbird Voice client placed in at initialisation.")
            .clone();

        let success = manager.join(guild_id, connect_to).await;

        if let Ok(handler_lock) = success {
            let evt_receiver = Receiver::new();
            let mut handler = handler_lock.lock().await;

            handler.add_global_event(CoreEvent::SpeakingStateUpdate.into(), evt_receiver.clone());
            handler.add_global_event(CoreEvent::RtpPacket.into(), evt_receiver.clone());
            handler.add_global_event(CoreEvent::RtcpPacket.into(), evt_receiver.clone());
            handler.add_global_event(CoreEvent::ClientDisconnect.into(), evt_receiver.clone());
            handler.add_global_event(CoreEvent::VoiceTick.into(), evt_receiver);

            let builder_embed

            return Ok(());
        } else if let Err(joining) = success {
            return Err(Box::new(FollowupError::Audio(format!(
                "Failed to join voice channel: {:#?}",
                joining
            ))));
        }
    }
}