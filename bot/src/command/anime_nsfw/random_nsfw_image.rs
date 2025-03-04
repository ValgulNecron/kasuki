use anyhow::{Result, anyhow};
use std::sync::Arc;

use crate::command::anime::random_image::send_embed;
use crate::command::command_trait::{Command, SlashCommand};
use crate::config::Config;
use crate::event_handler::BotData;
use crate::helper::get_option::subcommand::get_option_map_string_subcommand;
use crate::structure::message::anime_nsfw::random_image_nsfw::load_localization_random_image_nsfw;
use serenity::all::CreateInteractionResponse::Defer;
use serenity::all::{
	CommandInteraction, Context as SerenityContext, CreateInteractionResponseMessage,
};

pub struct AnimeRandomNsfwImageCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl Command for AnimeRandomNsfwImageCommand {
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}
}

impl SlashCommand for AnimeRandomNsfwImageCommand {
	async fn run_slash(&self) -> Result<()> {
		let ctx = self.get_ctx();
		let bot_data = ctx.data::<BotData>().clone();
		send(
			&self.ctx,
			&self.command_interaction,
			bot_data.config.clone(),
		)
		.await
	}
}

async fn send(
	ctx: &SerenityContext, command_interaction: &CommandInteraction, config: Arc<Config>,
) -> Result<()> {
	// Retrieve the type of image to fetch from the command interaction
	let map = get_option_map_string_subcommand(command_interaction);

	let image_type = map
		.get(&String::from("image_type"))
		.ok_or(anyhow!("No image type specified"))?;

	// Retrieve the guild ID from the command interaction
	let guild_id = match command_interaction.guild_id {
		Some(id) => id.to_string(),
		None => String::from("0"),
	};

	// Load the localized random NSFW image strings
	let random_image_nsfw_localised =
		load_localization_random_image_nsfw(guild_id, config.db.clone()).await?;

	// Create a deferred response to the command interaction
	let builder_message = Defer(CreateInteractionResponseMessage::new());

	// Send the deferred response
	command_interaction
		.create_response(&ctx.http, builder_message)
		.await?;

	// Send the random NSFW image as a response to the command interaction
	send_embed(
		ctx,
		command_interaction,
		image_type,
		random_image_nsfw_localised.title,
		"nsfw",
	)
	.await
}
