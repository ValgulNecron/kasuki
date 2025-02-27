use crate::command::command_trait::{Command, Embed, EmbedContent, EmbedType, SlashCommand};
use crate::event_handler::BotData;
use anyhow::{anyhow, Result};
use lavalink_rs::model::ChannelId;
use serenity::all::{CommandInteraction, Context as SerenityContext, Context};
use std::sync::Arc;
use serenity::http::Http;
use serenity::prelude::Mentionable;

pub struct JoinCommand {
    pub ctx: SerenityContext,
    pub command_interaction: CommandInteraction,
}

impl Command for JoinCommand {
    fn get_ctx(&self) -> &SerenityContext {
        &self.ctx
    }

    fn get_command_interaction(&self) -> &CommandInteraction {
        &self.command_interaction
    }
}

impl SlashCommand for JoinCommand {
    async fn run_slash(&self) -> Result<()> {
        let ctx = self.get_ctx();
        let bot_data = ctx.data::<BotData>().clone();
        let command_interaction = self.get_command_interaction();

        let (_, content) = join(ctx, bot_data, command_interaction).await?;

        self.send_embed(content).await
    }
}

pub async fn join<'a>(ctx: &'a Context, bot_data: Arc<BotData>, command_interaction: &'a CommandInteraction) -> Result<(bool, EmbedContent<'a, 'a>)> {
    let lava_client = bot_data.lavalink.read().await.clone();
    match lava_client {
        Some(_) => {}
        None => {
            return Err(anyhow::anyhow!("Lavalink is disabled"));
        }
    }
    let lava_client = lava_client.unwrap();
    let manager = bot_data.manager.clone();

    let guild_id = command_interaction.guild_id.ok_or(anyhow!("no guild id"))?;
    let guild = guild_id.to_guild_cached(&ctx.cache).ok_or(anyhow!("Guild not found"))?;
    let channel_id = command_interaction.channel_id;
    let author_id = command_interaction.user.id;

    let mut content = EmbedContent {
        title: "".to_string(),
        description: "".to_string(),
        thumbnail: None,
        url: None,
        command_type: EmbedType::Followup,
        colour: None,
        fields: vec![],
        images: None,
        action_row: None,
        images_url: None,
    };

    if lava_client.get_player_context(lavalink_rs::model::GuildId::from(guild_id.get())).is_none() {
        let connect_to =  {
                let user_channel_id = guild
                    .voice_states
                    .get(&author_id)
                    .and_then(|voice_state| voice_state.channel_id);

                match user_channel_id {
                    Some(channel) => channel,
                    None => {
                        content.description = "Not in a voice channel".to_string();
                        return Ok((false, content));
                    }
                }
            };


            let handler = manager.join_gateway(guild_id, connect_to).await;

        match handler {
            Ok((connection_info, _)) => {
                lava_client
                    .create_player_context_with_data::<(ChannelId, Arc<Http>)>(
                        lavalink_rs::model::GuildId::from(guild_id.get()),
                        lavalink_rs::model::player::ConnectionInfo {
                            endpoint: connection_info.endpoint,
                            token: connection_info.token,
                            session_id: connection_info.session_id,
                        },
                        Arc::new((
                            ChannelId(channel_id.get()),
                            ctx.http.clone(),
                        )),
                    )
                    .await?;

                content.description = format!("Joined {}", connect_to.mention());
                return Ok((true, content));
            }
            Err(why) => {
                content.description = format!("Error joining the channel: {}", why);
                return Ok((false, content));
            }
        }
    }
    return Ok((false, content));
}