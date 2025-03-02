use crate::command::command_trait::{Command, Embed, EmbedContent, EmbedType, SlashCommand};
use crate::event_handler::BotData;
use crate::structure::message::music::stop::load_localization_stop;
use anyhow::anyhow;
use serenity::all::{CommandInteraction, Context as SerenityContext};

pub struct StopCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl Command for StopCommand {
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}
}

impl SlashCommand for StopCommand {
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
		let stop_localised =
			load_localization_stop(guild_id_str, bot_data.config.db.clone()).await?;

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
				title: stop_localised.title,
				description: stop_localised.error_no_voice,
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
		let mut content = EmbedContent {
			title: stop_localised.title,
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

		let now_playing = player.get_player().await?.track;

		if let Some(np) = now_playing {
			player.stop_now().await?;
			content.description = stop_localised.success.replace("{0}", &np.info.title);
		} else {
			content.description = stop_localised.nothing_to_stop;
		}

		self.send_embed(content).await
	}
}
