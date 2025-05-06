use crate::command::command_trait::{Command, Embed, EmbedContent, EmbedType, SlashCommand};
use crate::event_handler::BotData;
use crate::structure::message::music::queue::load_localization_queue;
use anyhow::anyhow;
use futures::StreamExt;
use futures::future;
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

		// Retrieve the guild ID from the command interaction
		let guild_id_str = match self.command_interaction.guild_id {
			Some(id) => id.to_string(),
			None => String::from("0"),
		};

		// Load the localized strings
		let queue_localised =
			load_localization_queue(guild_id_str, bot_data.config.db.clone()).await?;

		let command_interaction = self.get_command_interaction();

		let guild_id = command_interaction.guild_id.ok_or(anyhow!("no guild id"))?;
		let lava_client = bot_data.lavalink.clone();
		let lava_client = lava_client.read().await.clone();
		if lava_client.is_none() {
			return Err(anyhow::anyhow!("Lavalink is disabled"));
		}
		let lava_client = lava_client.unwrap();
		let Some(player) =
			lava_client.get_player_context(lavalink_rs::model::GuildId::from(guild_id.get()))
		else {
			let content = EmbedContent::new(queue_localised.title)
				.description(queue_localised.error_no_voice)
				.command_type(EmbedType::Followup);
			return self.send_embed(vec![content]).await;
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
						"{} -> [{} - {}](<{}>) | {} <@!{}>",
						idx + 1,
						x.track.info.author,
						x.track.info.title,
						uri,
						queue_localised.requested_by,
						x.track.user_data.unwrap()["requester_id"]
					)
				} else {
					format!(
						"{} -> {} - {} | {} <@!{}",
						idx + 1,
						x.track.info.author,
						x.track.info.title,
						queue_localised.requested_by,
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
				queue_localised
					.now_playing
					.replace("{0}", &track.info.author)
					.replace("{1}", &track.info.title)
					.replace("{2}", uri)
					.replace("{3}", &time)
					.replace(
						"{4}",
						&format!("<@!{}>", track.user_data.unwrap()["requester_id"]),
					)
					.to_string()
			} else {
				queue_localised
					.now_playing
					.replace("{0}", &track.info.author)
					.replace("{1}", &track.info.title)
					.replace("{2}", "")
					.replace("{3}", &time)
					.replace(
						"{4}",
						&format!("<@!{}>", track.user_data.unwrap()["requester_id"]),
					)
					.to_string()
			}
		} else {
			queue_localised.nothing_playing.clone()
		};

		let content = EmbedContent::new(now_playing_message)
			.description(queue_message)
			.command_type(EmbedType::Followup);
		self.send_embed(vec![content]).await
	}
}
