use crate::command::command_trait::{Command, Embed, SlashCommand};
use crate::command::server::generate_image_pfp_server::get_content;
use crate::event_handler::BotData;
use anyhow::Result;
use serenity::all::{CommandInteraction, Context as SerenityContext};

pub struct GenerateGlobalImagePfPCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl Command for GenerateGlobalImagePfPCommand {
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}
}

impl SlashCommand for GenerateGlobalImagePfPCommand {
	async fn run_slash(&self) -> Result<()> {
		let ctx = self.get_ctx();
		let bot_data = ctx.data::<BotData>().clone();
		let command_interaction = self.get_command_interaction();
		let config = bot_data.config.clone();

		let content = get_content(ctx, command_interaction, "global", config.db.clone()).await?;

		self.send_embed(vec![content]).await
	}
}
