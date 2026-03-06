use anyhow::{anyhow, Result};
use fluent_templates::Loader;
use sea_orm::DatabaseConnection;
use serenity::all::{
	ComponentInteraction, Context as SerenityContext, CreateButton, EditMessage, UserId,
};
use shared::helper::get_guild_lang::get_guild_language;
use shared::localization::USABLE_LOCALES;
use std::future::Future;
use std::pin::Pin;
use std::str::FromStr;
use std::sync::Arc;
use unic_langid::LanguageIdentifier;

use crate::command::anilist_server::list_register_user::get_the_list;
use crate::components::handler::ComponentHandler;
use crate::constant::MEMBER_LIST_LIMIT;
use crate::event_handler::BotData;
use crate::helper::create_default_embed::get_default_embed;

pub async fn update(
	ctx: &SerenityContext, component_interaction: &ComponentInteraction, user_id: &str,
	prev_id: &str,
) -> Result<()> {
	let bot_data = ctx.data::<BotData>().clone();
	let connection = bot_data.db_connection.clone();
	// Retrieve the guild ID from the component interaction
	let guild_id = match component_interaction.guild_id {
		Some(id) => id.to_string(),
		None => String::from("0"),
	};
	let db_connection = bot_data.db_connection.clone();

	// Load the localized user list
	let lang = get_guild_language(guild_id.clone(), db_connection).await;
	let lang_code = match lang.as_str() {
		"jp" => "ja",
		"en" => "en-US",
		other => other,
	};
	let lang_id = LanguageIdentifier::from_str(lang_code)
		.unwrap_or_else(|_| LanguageIdentifier::from_str("en-US").unwrap());
	let previous = USABLE_LOCALES.lookup(&lang_id, "anilist_server_list_register_user-previous");
	let next = USABLE_LOCALES.lookup(&lang_id, "anilist_server_list_register_user-next");

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

	let embed = get_default_embed(None, &None)
		.title(old_embed_title)
		.description(builder_message);

	// Create the response message
	let mut response = EditMessage::new().embed(embed);

	if user_id != "0" {
		response = response
			.button(CreateButton::new(format!("user_{}_{}", user_id, prev_id)).label(&previous));
	}

	if len > MEMBER_LIST_LIMIT as usize {
		response = response
			.button(CreateButton::new(format!("user_{}_{}", last_id.unwrap(), user_id)).label(next))
	}

	// Clone the component interaction message
	let mut message = component_interaction.message.clone();

	// Edit the message with the response
	message.edit(&ctx.http, response).await?;

	Ok(())
}

pub struct ListRegisterUserHandler;

impl ComponentHandler for ListRegisterUserHandler {
	fn prefix(&self) -> &'static str {
		"user_"
	}

	fn handle<'a>(
		&'a self, ctx: &'a SerenityContext, interaction: &'a ComponentInteraction,
		_db: Arc<DatabaseConnection>,
	) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>> {
		Box::pin(async move {
			let s = interaction.data.custom_id.as_str();
			let user_id = s.split_at("_".len()).1;
			let prev_id = user_id.split_at("_".len()).1;
			update(ctx, interaction, user_id, prev_id).await
		})
	}
}

inventory::submit! { &ListRegisterUserHandler as &dyn ComponentHandler }
