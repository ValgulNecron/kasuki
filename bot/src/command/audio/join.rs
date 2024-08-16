use crate::audio::receiver::Receiver;
use crate::command::command_trait::{Command, SlashCommand};
use crate::config::Config;
use crate::helper::create_default_embed::get_default_embed;
use crate::helper::error_management::error_enum::ResponseError;
use crate::structure::message::audio::join::load_localization_join_localised;
use serenity::all::{CommandInteraction, Context, CreateEmbed};
use serenity::builder::{CreateInteractionResponse, CreateInteractionResponseMessage};
use songbird::CoreEvent;
use std::error::Error;
use std::sync::Arc;
use tracing::{error, trace};

pub struct AudioJoinCommand {
    pub ctx: Context,
    pub command_interaction: CommandInteraction,
    pub config: Arc<Config>,
}

impl Command for AudioJoinCommand {
    fn get_command_interaction(&self) -> &CommandInteraction {
        &self.command_interaction
    }

    fn get_ctx(&self) -> &Context {
        &self.ctx
    }
}

impl SlashCommand for AudioJoinCommand {
    async fn run_slash(&self) -> Result<(), Box<dyn Error>> {
        send_embed(&self.ctx, &self.command_interaction, self.config.clone()).await
    }
}
async fn send_embed(
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
    let localised =
        load_localization_join_localised(guild_id.to_string(), config.bot.config.clone()).await?;

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
                return Err(Box::new(ResponseError::Option(String::from(
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
            let embed = CreateEmbed::new().title(localised.title);
            let builder_embed = CreateInteractionResponseMessage::new().embed(embed);
            let builder = CreateInteractionResponse::Message(builder_embed);
            command_interaction
                .create_response(&ctx.http, builder)
                .await
                .map_err(|e| ResponseError::Sending(format!("{:#?}", e)))?;

            return Ok(());
        } else if let Err(joining) = success {
            return Err(Box::new(ResponseError::Audio(format!(
                "Failed to join voice channel: {:#?}",
                joining
            ))));
        }
        Ok(())
    } else {
        let embed = get_default_embed(None).title(localised.already_in);
        let builder_embed = CreateInteractionResponseMessage::new().embed(embed);
        let builder = CreateInteractionResponse::Message(builder_embed);
        command_interaction
            .create_response(&ctx.http, builder)
            .await
            .map_err(|e| ResponseError::Sending(format!("{:#?}", e)))?;

        Ok(())
    }
}
