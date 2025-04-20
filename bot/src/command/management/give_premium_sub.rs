use anyhow::{Result, anyhow};

use crate::command::command_trait::{Command, Embed, EmbedContent, SlashCommand};
use crate::event_handler::BotData;
use crate::helper::get_option::command::{get_option_map_string, get_option_map_user};
use crate::structure::message::management::give_premium_sub::load_localization_give_premium_sub;
use serenity::all::{CommandInteraction, Context as SerenityContext, EntitlementOwner};
use small_fixed_array::FixedString;

pub struct GivePremiumSubCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl Command for GivePremiumSubCommand {
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}
}

impl SlashCommand for GivePremiumSubCommand {
	async fn run_slash(&self) -> Result<()> {
		let ctx = self.get_ctx();
		let bot_data = ctx.data::<BotData>().clone();
		let command_interaction = self.get_command_interaction();
		let config = &bot_data.config;

		let map = get_option_map_user(command_interaction);

		let user = *map
			.get(&FixedString::from_str_trunc("user"))
			.ok_or(anyhow!("No option for user"))?;

		let map = get_option_map_string(command_interaction);

		let subscription = map
			.get(&FixedString::from_str_trunc("subscription"))
			.ok_or(anyhow!("No option for subscription"))?
			.clone();

		let skus = ctx.http.get_skus().await?;

		let skus_id: Vec<String> = skus.iter().map(|sku| sku.id.to_string()).collect();

		if !skus_id.contains(&subscription) {
			Err(anyhow!("Invalid sub id"))?
		}

		let mut sku_id = Default::default();

		for sku in skus {
			if sku.id.to_string() == subscription {
				sku_id = sku.id;
			}
		}

		let _ = ctx
			.http
			.create_test_entitlement(sku_id, EntitlementOwner::User(user))
			.await?;

		let localization = load_localization_give_premium_sub(
			command_interaction.guild_id.unwrap().to_string(),
			config.db.clone(),
		)
		.await?;

		let content = EmbedContent::new(String::default()).description(
			localization
				.success
				.replace("{user}", &user.to_string())
				.replace("{subscription}", &subscription),
		);

		self.send_embed(vec![content]).await
	}
}
