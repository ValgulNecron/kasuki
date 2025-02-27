use crate::command::command_trait::{Command, Embed, EmbedContent, EmbedType, SlashCommand};
use crate::event_handler::BotData;
use anyhow::anyhow;
use futures::future;
use futures::StreamExt;
use lavalink_rs::client::LavalinkClient;
use serenity::all::{CommandInteraction, Context as SerenityContext};

pub struct QueueCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl Command for QueueCommand {
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}
}

impl SlashCommand for QueueCommand {
	async fn run_slash(&self) -> anyhow::Result<()> {
		let ctx = self.get_ctx();
		let bot_data = ctx.data::<BotData>().clone();

		self.defer().await?;
		let command_interaction = self.get_command_interaction();

		let guild_id = command_interaction.guild_id.ok_or(anyhow!("no guild id"))?;
		let lava_client = bot_data.lavalink.clone();
		let lava_client = lava_client.read().await.clone();
		match lava_client {
			None => {
				return Err(anyhow::anyhow!("Lavalink is disabled"));
			},
			_ => {},
		}
		let lava_client = lava_client.unwrap();
		let Some(player) =
			lava_client.get_player_context(lavalink_rs::model::GuildId::from(guild_id.get()))
		else {
			let content = EmbedContent {
				title: "".to_string(),
				description: "Join the bot to a voice channel first.".to_string(),
				thumbnail: None,
				url: None,
				command_type: EmbedType::Followup,
				colour: None,
				fields: vec![],
				images: None,
				action_row: None,
				images_url: None,
			};
			return self.send_embed(content).await;
		};
		let queue = player.get_queue();
		let player_data = player.get_player().await?;

		let max = queue.get_count().await?.min(9);

		let queue_message = queue
			.enumerate()
			.take_while(|(idx, _)| future::ready(*idx < max))
			.map(|(idx, x)| {
				if let Some(uri) = &x.track.info.uri {
					format!(
						"{} -> [{} - {}](<{}>) | Requested by <@!{}>",
						idx + 1,
						x.track.info.author,
						x.track.info.title,
						uri,
						x.track.user_data.unwrap()["requester_id"]
					)
				} else {
					format!(
						"{} -> {} - {} | Requested by <@!{}",
						idx + 1,
						x.track.info.author,
						x.track.info.title,
						x.track.user_data.unwrap()["requester_id"]
					)
				}
			})
			.collect::<Vec<_>>()
			.await
			.join("\n");

		let now_playing_message = if let Some(track) = player_data.track {
			let time_s = player_data.state.position / 1000 % 60;
			let time_m = player_data.state.position / 1000 / 60;
			let time = format!("{:02}:{:02}", time_m, time_s);

			if let Some(uri) = &track.info.uri {
				format!(
					"Now playing: [{} - {}](<{}>) | {}, Requested by <@!{}>",
					track.info.author,
					track.info.title,
					uri,
					time,
					track.user_data.unwrap()["requester_id"]
				)
			} else {
				format!(
					"Now playing: {} - {} | {}, Requested by <@!{}>",
					track.info.author,
					track.info.title,
					time,
					track.user_data.unwrap()["requester_id"]
				)
			}
		} else {
			"Now playing: nothing".to_string()
		};

		let content = EmbedContent {
			title: now_playing_message,
			description: queue_message,
			thumbnail: None,
			url: None,
			command_type: EmbedType::Followup,
			colour: None,
			fields: vec![],
			images: None,
			action_row: None,
			images_url: None,
		};
		self.send_embed(content).await
	}
}
