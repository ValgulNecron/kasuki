use crate::audio::receiver::{Receiver, TrackErrorNotifier};
use crate::config::Config;
use crate::helper::create_default_embed::get_default_embed;
use crate::helper::error_management::error_enum::{FollowupError, ResponseError};
use crate::helper::get_option::command::get_option_map_string;
use crate::helper::get_option::subcommand::get_option_map_string_subcommand;
use serenity::all::{CommandInteraction, CreateInteractionResponseFollowup};
use serenity::builder::CreateInteractionResponse::Defer;
use serenity::builder::CreateInteractionResponseMessage;
use serenity::client::Context;
use songbird::input::{Compose, YoutubeDl};
use songbird::tracks::Track;
use songbird::{CoreEvent, Event, TrackEvent};
use std::error::Error;
use std::sync::Arc;
use tracing::{error, trace};

pub async fn run(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    config: Arc<Config>,
) -> Result<(), Box<dyn Error>> {
    let map = get_option_map_string_subcommand(command_interaction);
    let mut url = map
        .get(&String::from("song"))
        .ok_or(ResponseError::Option(String::from("No option for song")))?
        .clone();

    let guild_id = command_interaction
        .guild_id
        .ok_or(ResponseError::Option(String::from("No guild id")))?;
    let cache = ctx.cache.clone();

    let http_client = reqwest::Client::new();
    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();
    trace!(?manager);

    let builder_message = Defer(CreateInteractionResponseMessage::new());

    command_interaction
        .create_response(&ctx.http, builder_message)
        .await
        .map_err(|e| ResponseError::Sending(format!("{:#?}", e)))?;
    let bind = manager.get(guild_id);
    trace!(?bind);

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
        } else if let Err(joining) = success {
            return Err(Box::new(FollowupError::Audio(format!(
                "Failed to join voice channel: {:#?}",
                joining
            ))));
        }
    }
    let bind = manager.get(guild_id);
    trace!(?bind);
    if url.clone().starts_with("http") && url.contains("music.") {
        url = url.replace("music.", "");
    }
    if let Some(handler_lock) = bind {
        let mut handler = handler_lock.lock().await;

        let do_search = !url.starts_with("http");
        let mut src = if do_search {
            YoutubeDl::new_search(http_client, url.clone())
        } else {
            YoutubeDl::new(http_client, url.clone())
        };
        let (track, meta) = futures::join!(
            handler.enqueue(Track::from(src.clone())),
            src.aux_metadata()
        );
        handler.add_global_event(TrackEvent::Error.into(), TrackErrorNotifier);
        trace!(?track, ?meta);
        if let Ok(meta) = meta {
            let title = meta.title.unwrap_or("song".to_string());
            let source_url = meta
                .source_url
                .unwrap_or("<https://youtube.com>".to_string());

            let embed = get_default_embed(None).title("Audio").description(format!(
                "Now playing: [{}]({})",
                title,
                url.clone()
            ));
            let builder = CreateInteractionResponseFollowup::new().embed(embed);
            command_interaction
                .create_followup(&ctx.http, builder)
                .await
                .map_err(|e| FollowupError::Sending(format!("{:#?}", e)))?;
            return Ok(());
        }
    }

    let embed = get_default_embed(None)
        .title("Audio")
        .description("No audio is playing");
    let builder = CreateInteractionResponseFollowup::new().embed(embed);
    command_interaction
        .create_followup(&ctx.http, builder)
        .await
        .map_err(|e| FollowupError::Sending(format!("{:#?}", e)))?;
    Ok(())
}
