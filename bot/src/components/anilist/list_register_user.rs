use anyhow::{Result, anyhow};

use serenity::all::{
	ComponentInteraction, Context as SerenityContext, CreateButton, EditMessage, UserId,
};

use crate::command::anilist_server::list_register_user::get_the_list;
use crate::config::DbConfig;
use crate::constant::MEMBER_LIST_LIMIT;
use crate::event_handler::BotData;
use crate::helper::create_default_embed::get_default_embed;
use crate::structure::message::anilist_server::list_register_user::load_localization_list_user;

pub async fn update(
	ctx: &SerenityContext, component_interaction: &ComponentInteraction, user_id: &str,
	prev_id: &str, db_config: DbConfig,
) -> Result<()> {
	let bot_data = ctx.data::<BotData>().clone();
	let connection = bot_data.db_connection.clone();
	// Retrieve the guild ID from the component interaction
	let guild_id = match component_interaction.guild_id {
		Some(id) => id.to_string(),
		None => String::from("0"),
	};

	// Load the localized user list
	let list_user_localised = load_localization_list_user(guild_id, db_config.clone()).await?;

	// Retrieve the guild ID from the component interaction
	let guild_id = component_interaction
		.guild_id
		.ok_or(anyhow!("Guild ID not found"))?;

	// Retrieve the guild with counts
	let guild = guild_id.to_partial_guild_with_counts(&ctx.http).await?;

	// Parse the user ID
	let id = if user_id == "0" {
		None
	} else {
		user_id.parse().ok()
	};

	// Get the list of users
	let (builder_message, len, last_id): (String, usize, Option<UserId>) =
		get_the_list(guild, ctx, id, connection).await?;

	let old_embed_title = component_interaction
		.message
		.embeds
		.first()
		.and_then(|embed| embed.title.clone())
		.unwrap_or_default();

	let embed = get_default_embed(None)
		.title(old_embed_title)
		.description(builder_message);

	// Create the response message
	let mut response = EditMessage::new().embed(embed);

	if user_id != "0" {
		response = response.button(
			CreateButton::new(format!("user_{}_{}", user_id, prev_id))
				.label(&list_user_localised.previous),
		);
	}

	if len > MEMBER_LIST_LIMIT as usize {
		response = response.button(
			CreateButton::new(format!("user_{}_{}", last_id.unwrap(), user_id))
				.label(list_user_localised.next),
		)
	}

	// Clone the component interaction message
	let mut message = component_interaction.message.clone();

	// Edit the message with the response
	message.edit(&ctx.http, response).await?;

	Ok(())
}
