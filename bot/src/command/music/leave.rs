use crate::command::command_trait::{Command, Embed, EmbedContent, EmbedType, SlashCommand};
use crate::event_handler::BotData;
use crate::structure::message::music::leave::load_localization_leave;
use anyhow::anyhow;
use serenity::all::{CommandInteraction, Context as SerenityContext};

pub struct LeaveCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl Command for LeaveCommand {
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}
}

impl SlashCommand for LeaveCommand {
	async fn run_slash(&self) -> anyhow::Result<()> {
		let ctx = self.get_ctx();
		let bot_data = ctx.data::<BotData>().clone();
		let command_interaction = self.get_command_interaction();
		self.defer().await?;

		// Retrieve the guild ID from the command interaction
		let guild_id_str = match command_interaction.guild_id {
			Some(id) => id.to_string(),
			None => String::from("0"),
		};

		// Load the localized strings
		let leave_localised =
			load_localization_leave(guild_id_str, bot_data.config.db.clone()).await?;

		let manager = bot_data.manager.clone();
		let lava_client = bot_data.lavalink.clone();
		let lava_client = lava_client.read().await.clone();
		if lava_client.is_none() {
			return Err(anyhow::anyhow!("Lavalink is disabled"));
		}
		let guild_id = command_interaction.guild_id.ok_or(anyhow!("no guild id"))?;

		let lava_client = lava_client.unwrap();

		lava_client
			.delete_player(lavalink_rs::model::GuildId::from(guild_id.get()))
			.await?;

		if manager.get(guild_id).is_some() {
			manager.remove(guild_id).await?;
		}

		let mut content = EmbedContent::new(leave_localised.title)
			.description(leave_localised.success)
			.command_type(EmbedType::Followup);

		self.send_embed(vec![content]).await
	}
}
