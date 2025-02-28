use crate::command::command_trait::{Command, SlashCommand};
use crate::config::Config;
use crate::event_handler::BotData;
use crate::helper::create_default_embed::get_default_embed;
use crate::helper::get_option::command::get_option_map_user;
use crate::structure::message::management::remove_test_sub::load_localization_remove_test_sub;
use anyhow::{Result, anyhow};
use serenity::all::CreateInteractionResponse::Defer;
use serenity::all::{
	CommandInteraction, Context as SerenityContext, CreateInteractionResponseFollowup,
	CreateInteractionResponseMessage,
};
use small_fixed_array::FixedString;
use std::sync::Arc;
use tracing::error;

pub struct RemoveTestSubCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl Command for RemoveTestSubCommand {
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}
}

impl SlashCommand for RemoveTestSubCommand {
	async fn run_slash(&self) -> Result<()> {
		let ctx = self.get_ctx();
		let bot_data = ctx.data::<BotData>().clone();
		send_embed(
			&self.ctx,
			&self.command_interaction,
			bot_data.config.clone(),
		)
		.await
	}
}

async fn send_embed(
	ctx: &SerenityContext, command_interaction: &CommandInteraction, config: Arc<Config>,
) -> Result<()> {
	let map = get_option_map_user(command_interaction);

	let user = map.get(&FixedString::from_str_trunc("user"));

	let user = match user {
		Some(user) => user,
		None => {
			return Err(anyhow!("No user provided"));
		},
	};

	let entitlements = ctx
		.http
		.get_entitlements(Some(*user), None, None, None, None, None, None)
		.await?;

	let localization = load_localization_remove_test_sub(
		command_interaction.guild_id.unwrap().to_string(),
		config.db.clone(),
	)
	.await?;

	// defer the response
	let builder_message = Defer(CreateInteractionResponseMessage::new());

	command_interaction
		.create_response(&ctx.http, builder_message)
		.await?;

	for entitlement in entitlements {
		if let Err(e) = ctx.http.delete_test_entitlement(entitlement.id).await {
			error!("Error while deleting entitlement: {}", e);
		}
	}

	let embed = get_default_embed(None)
		.description(localization.success.replace("{user}", &user.to_string()));

	let builder = CreateInteractionResponseFollowup::new().embed(embed);

	command_interaction
		.create_followup(&ctx.http, builder)
		.await?;

	Ok(())
}
