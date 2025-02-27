use crate::command::command_trait::{Command, Embed, EmbedContent, EmbedType, SlashCommand};
use crate::event_handler::BotData;
use anyhow::anyhow;
use lavalink_rs::client::LavalinkClient;
use serenity::all::{CommandInteraction, Context as SerenityContext};

pub struct ResumeCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl Command for ResumeCommand {
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}
}

impl SlashCommand for ResumeCommand {
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

		player.set_pause(false).await?;

		let content = EmbedContent {
			title: "".to_string(),
			description: "Resumed playback".to_string(),
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
