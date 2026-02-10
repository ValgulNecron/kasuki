use crate::constant::{ACTIVITY_LIST_LIMIT, COLOR};
use anyhow::{anyhow, Result};
use fluent_templates::Loader;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serenity::all::{
	ComponentInteraction, Context as SerenityContext, CreateButton, CreateEmbed,
	CreateInteractionResponse, CreateInteractionResponseMessage, Timestamp,
};
use shared::database::activity_data::{Column, Model};
use shared::database::prelude::ActivityData;
use shared::helper::get_guild_lang::get_guild_language;
use shared::localization::USABLE_LOCALES;
use std::str::FromStr;
use std::sync::Arc;
use tracing::trace;
use unic_langid::LanguageIdentifier;

pub async fn update(
	ctx: &SerenityContext, component_interaction: &ComponentInteraction, page_number: &str,
	db_connection: Arc<DatabaseConnection>,
) -> Result<()> {
	let guild_id = match component_interaction.guild_id {
		Some(id) => id.to_string(),
		None => String::from("0"),
	};

	let lang = get_guild_language(guild_id, db_connection.clone()).await;
	let lang_code = match lang.as_str() {
		"jp" => "ja",
		"en" => "en-US",
		other => other,
	};
	let lang_id = LanguageIdentifier::from_str(lang_code)
		.unwrap_or_else(|_| LanguageIdentifier::from_str("en-US").unwrap());
	let title = USABLE_LOCALES.lookup(&lang_id, "anilist_server_list_all_activity-title");
	let previous = USABLE_LOCALES.lookup(&lang_id, "anilist_server_list_all_activity-previous");
	let next = USABLE_LOCALES.lookup(&lang_id, "anilist_server_list_all_activity-next");

	let guild_id = component_interaction
		.guild_id
		.ok_or(anyhow!("Guild ID not found"))?;

	let list = ActivityData::find()
		.filter(Column::ServerId.eq(guild_id.to_string()))
		.all(&*db_connection)
		.await?;

	let len = list.len();

	let actual_page: u64 = page_number.parse()?;

	trace!("{:?}", actual_page);

	let next_page: u64 = actual_page + 1;

	let previous_page: u64 = if actual_page > 0 { actual_page - 1 } else { 0 };

	let activity: Vec<String> = get_formatted_activity_list(list, actual_page);

	let join_activity = activity.join("\n");

	let builder_message = CreateEmbed::new()
		.timestamp(Timestamp::now())
		.color(COLOR)
		.title(title)
		.description(join_activity);

	let mut message_rep = CreateInteractionResponseMessage::new().embed(builder_message);

	if page_number != "0" {
		message_rep = message_rep
			.button(CreateButton::new(format!("next_activity_{}", previous_page)).label(&previous));
	}

	trace!("{:?}", len);

	trace!("{:?}", ACTIVITY_LIST_LIMIT);

	if len > ACTIVITY_LIST_LIMIT as usize
		&& (len > (ACTIVITY_LIST_LIMIT * (actual_page + 1)) as usize)
	{
		message_rep = message_rep
			.button(CreateButton::new(format!("next_activity_{}", next_page)).label(&next))
	}

	let response = CreateInteractionResponse::UpdateMessage(message_rep);

	component_interaction
		.create_response(&ctx.http, response)
		.await?;

	Ok(())
}

pub fn get_formatted_activity_list(list: Vec<Model>, actual_page: u64) -> Vec<String> {
	list.into_iter()
		.map(|activity| {
			let anime_id = activity.anime_id;

			let name = activity.name;

			format!("[{}](https://anilist_user.co/anime/{})", name, anime_id)
		})
		.skip((ACTIVITY_LIST_LIMIT * actual_page) as usize)
		.take(ACTIVITY_LIST_LIMIT as usize)
		.collect()
}
