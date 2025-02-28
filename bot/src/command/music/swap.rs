use crate::command::command_trait::{Command, Embed, EmbedContent, EmbedType, SlashCommand};
use crate::event_handler::BotData;
use crate::helper::get_option::command::get_option_map_number;
use crate::helper::get_option::subcommand::get_option_map_number_subcommand;
use crate::structure::message::music::swap::load_localization_swap;
use anyhow::anyhow;
use lavalink_rs::client::LavalinkClient;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use small_fixed_array::FixedString;
use std::ops::Deref;

pub struct SwapCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl Command for SwapCommand {
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}
}

impl SlashCommand for SwapCommand {
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
		let swap_localised =
			load_localization_swap(guild_id_str, bot_data.config.db.clone()).await?;

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
				title: swap_localised.title,
				description: swap_localised.error_no_voice,
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

		let map = get_option_map_number_subcommand(command_interaction);

		let index1 = map
			.get(&String::from("index1"))
			.cloned()
			.unwrap_or_default() as usize;

		let index2 = map
			.get(&String::from("index2"))
			.cloned()
			.unwrap_or_default() as usize;

		let mut content = EmbedContent {
			title: swap_localised.title,
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

		let queue = player.get_queue();
		let queue_len = queue.get_count().await?;

		if index1 > queue_len || index2 > queue_len {
			content.description = swap_localised
				.error_max_index
				.replace("{0}", &queue_len.to_string());
			self.send_embed(content).await
		} else if index1 == index2 {
			content.description = swap_localised.error_same_index;
			self.send_embed(content).await
		} else {
			let track1 = queue.get_track(index1 - 1).await?.unwrap();
			let track2 = queue.get_track(index1 - 2).await?.unwrap();

			queue.swap(index1 - 1, track2)?;
			queue.swap(index2 - 1, track1)?;

			content.description = swap_localised.success;

			self.send_embed(content).await
		}
	}
}
