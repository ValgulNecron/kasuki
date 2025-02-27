use crate::command::command_trait::{Command, Embed, EmbedContent, EmbedType, SlashCommand};
use crate::event_handler::BotData;
use anyhow::anyhow;
use lavalink_rs::player_context::TrackInQueue;
use lavalink_rs::prelude::{SearchEngines, TrackLoadData};
use serenity::all::{CommandInteraction, Context as SerenityContext};
use small_fixed_array::FixedString;
use crate::command::music::join::join;
use crate::helper::get_option::command::{get_option_map_number, get_option_map_string};

pub struct PlayCommand {
    pub ctx: SerenityContext,
    pub command_interaction: CommandInteraction,
}

impl Command for PlayCommand {
    fn get_ctx(&self) -> &SerenityContext {
        &self.ctx
    }

    fn get_command_interaction(&self) -> &CommandInteraction {
        &self.command_interaction
    }
}

impl SlashCommand for PlayCommand {
    async fn run_slash(&self) -> anyhow::Result<()> {
        let ctx = self.get_ctx();
        let bot_data = ctx.data::<BotData>().clone();
        let command_interaction = self.get_command_interaction();
        let lava_client = bot_data.lavalink.read().await.clone();
        let (has_joined, mut content) = join(ctx, bot_data, command_interaction).await?;

        match lava_client {
            Some(_) => {}
            None => {
                return Err(anyhow::anyhow!("Lavalink is disabled"));
            }
        }
        let lava_client = lava_client.unwrap();
        let command_interaction = self.get_command_interaction();
        let guild_id = command_interaction.guild_id.ok_or(anyhow!("no guild id"))?;

        let Some(player) = lava_client.get_player_context(lavalink_rs::model::GuildId::from(guild_id.get())) else {
            ctx.say("Join the bot to a voice channel first.").await?;
            return Ok(());
        };

        let map = get_option_map_string(command_interaction);

        let term = map
            .get(&FixedString::from_str_trunc("search"))
            .cloned()
            .unwrap_or_default();

        let query = if let Some(term) = term {
            if term.starts_with("http") {
                term
            } else {
                //SearchEngines::YouTube.to_query(&term)?
                SearchEngines::YouTube.to_query(&term)?
            }
        } else {
            if let Ok(player_data) = player.get_player().await {
                let queue = player.get_queue();

                if player_data.track.is_none() && queue.get_track(0).await.is_ok_and(|x| x.is_some()) {
                    player.skip()?;
                } else {
                    content.description = "The queue is empty.".to_string();
                    self.send_embed(content).await?;
                }
            }

            return Ok(());
        };

        let loaded_tracks = lava_client.load_tracks(guild_id, &query).await?;

        let mut playlist_info = None;


        let mut tracks: Vec<TrackInQueue> = match loaded_tracks.data {
            Some(TrackLoadData::Track(x)) => vec![x.into()],
            Some(TrackLoadData::Search(x)) => vec![x[0].clone().into()],
            Some(TrackLoadData::Playlist(x)) => {
                playlist_info = Some(x.info);
                x.tracks.iter().map(|x| x.clone().into()).collect()
            }

            _ => {
                content.description = format!("{:?}", loaded_tracks);
                self.send_embed(content).await?;
                return Ok(());
            }
        };

        if let Some(info) = playlist_info {
            ctx.say(format!("Added playlist to queue: {}", info.name,))
                .await?;
        } else {
            let track = &tracks[0].track;

            if let Some(uri) = &track.info.uri {
                content.description = format!(
                    "Added to queue: [{} - {}](<{}>)",
                    track.info.author, track.info.title, uri
                );
                self.send_embed(content).await?;
                return Ok(());
            } else {
                content.description = format!(
                    "Added to queue: {} - {}",
                    track.info.author, track.info.title
                );
                self.send_embed(content).await?;
                return Ok(());
            }
        }

        for i in &mut tracks {
            i.track.user_data = Some(serde_json::json!({"requester_id": ctx.author().id.get()}));
        }

        let queue = player.get_queue();
        queue.append(tracks.into())?;

        if has_joined {
            return Ok(());
        }

        if let Ok(player_data) = player.get_player().await {
            if player_data.track.is_none() && queue.get_track(0).await.is_ok_and(|x| x.is_some()) {
                player.skip()?;
            }
        }

        Ok(())

    }
}