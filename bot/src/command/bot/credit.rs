use anyhow::Result;

use crate::command::command_trait::{Command, Embed, EmbedContent, EmbedType, SlashCommand};
use crate::event_handler::BotData;
use crate::structure::message::bot::credit::load_localization_credit;
use serenity::all::{CommandInteraction, Context as SerenityContext};

pub struct CreditCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl Command for CreditCommand {
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}
}

impl SlashCommand for CreditCommand {
	async fn run_slash(&self) -> Result<()> {
		let ctx = self.get_ctx();
		let bot_data = ctx.data::<BotData>().clone();
		let command_interaction = self.get_command_interaction();
		let config = bot_data.config.clone();

		// Retrieve the guild ID from the command interaction or use "0" if it does not exist
		let guild_id = match command_interaction.guild_id {
			Some(id) => id.to_string(),
			None => String::from("0"),
		};

		// Load the localized strings for the credits
		let credit_localised = load_localization_credit(guild_id, config.db.clone()).await?;

		// Construct a description by concatenating the descriptions of all credits
		let mut desc: String = "".to_string();

		for x in credit_localised.credits {
			desc += x.desc.as_str()
		}

		let content = EmbedContent::new(credit_localised.title)
			.description(desc)
			.command_type(EmbedType::First);

		self.send_embed(vec![content]).await
	}
}
