use crate::command::command_trait::{Command, Embed, EmbedContent, EmbedType, SlashCommand};
use crate::event_handler::BotData;
use anyhow::{anyhow, Result};
use lavalink_rs::http::Http;
use lavalink_rs::model::ChannelId;
use serenity::all::{CommandInteraction, Context as SerenityContext, Context};
use std::sync::Arc;

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

        let (_, content) = join(ctx, bot_data, command_interaction);

        self.send_embed(content)
    }
}

pub async fn join(ctx: &Context, bot_data: Arc<BotData>, command_interaction: &CommandInteraction) -> Result<(bool, EmbedContent)> {
    let lava_client = ctx.data().lavalink.clone();

    let manager = bot_data.manager.clone();

    let guild_id = command_interaction.guild_id.ok_or(anyhow!("no guild id"))?;
    let guild = guild_id.to_guild_cached(ctx).await?;
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

    if lava_client.get_player_context(guild_id).is_none() {
        let connect_to = match channel_id {
            Some(x) => x,
            None => {
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
            }
        };

        let handler = manager.join_gateway(guild_id, connect_to).await;

        match handler {
            Ok((connection_info, _)) => {
                lava_client
                    .create_player_context_with_data::<(ChannelId, Arc<Http>)>(
                        guild_id,
                        connection_info,
                        Arc::new((
                            ctx.channel_id(),
                            ctx.serenity_context().http.clone(),
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
}