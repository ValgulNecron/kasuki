//! This module implements functionality to manage the deletion of activities related to anime
//! within a Discord bot using the Serenity and SeaORM libraries. It defines the `DeleteActivityCommand`,
//! enabling the ability to delete activities associated with an anime, and the helper function `remove_activity` to handle database operations.
use crate::command::admin::anilist::add_activity::{get_minimal_anime_media, get_name};
use crate::command::command::CommandRun;
use crate::command::embed_content::{EmbedContent, EmbedsContents};
use crate::event_handler::BotData;
use crate::get_url;
use crate::helper::get_option::subcommand_group::get_option_map_string_subcommand_group;
use anyhow::{anyhow, Result};
use fluent_templates::fluent_bundle::FluentValue;
use fluent_templates::Loader;
use kasuki_macros::slash_command;
use sea_orm::ColumnTrait;
use sea_orm::{EntityTrait, ModelTrait, QueryFilter};
use serenity::all::{CommandInteraction, Context as SerenityContext};
use shared::config::DbConfig;
use shared::database::prelude::ActivityData;
use shared::localization::{get_language_identifier, USABLE_LOCALES};
use std::borrow::Cow;
use std::collections::HashMap;

#[slash_command(
	name = "delete_anime_activity", desc = "Delete an anime activity.",
	command_type = SubCommandGroup(parent = "admin", group = "anilist"),
	args = [(name = "anime_name", desc = "Name of the anime you want to delete as an activity.", arg_type = String, required = true, autocomplete = true)],
)]
async fn delete_activity_command(self_: DeleteActivityCommand) -> Result<EmbedsContents<'_>> {
	let command_interaction = self_.get_command_interaction();
	let ctx = self_.get_ctx();
	let bot_data = ctx.data::<BotData>().clone();
	let config = bot_data.config.clone();
	let anilist_cache = bot_data.anilist_cache.clone();

	let map = get_option_map_string_subcommand_group(&command_interaction);
	let anime = map
		.get(&String::from("anime_name"))
		.cloned()
		.unwrap_or(String::new());

	let guild_id = match command_interaction.guild_id {
		Some(id) => id.to_string(),
		None => String::from("1"),
	};
	let db_connection = bot_data.db_connection.clone();

	let lang_id = get_language_identifier(guild_id.clone(), db_connection).await;
	let media = get_minimal_anime_media(anime.to_string(), anilist_cache);


	let media = media.await?;
	let anime_id = media.id;

	remove_activity(guild_id.as_str(), &anime_id, config.db.clone()).await?;

	let title = media
		.title
		.ok_or(anyhow!(format!("Anime with id {} not found", anime_id)))?;
	let anime_name = get_name(title);

	let url = format!("https://anilist.co/anime/{}", anime_id);

	let mut args = HashMap::new();
	args.insert(
		Cow::Borrowed("anime"),
		FluentValue::from(anime_name.as_str()),
	);

	let embed_content =
		EmbedContent::new(USABLE_LOCALES.lookup(&lang_id, "admin_anilist_delete_activity-success"))
			.description(USABLE_LOCALES.lookup_with_args(
				&lang_id,
				"admin_anilist_delete_activity-success_desc",
				&args,
			))
			.url(url);

	let embed_contents = EmbedsContents::new(vec![embed_content]);

	Ok(embed_contents)
}

/// Asynchronously removes an activity entry from the database based on the provided guild ID and anime ID.
///
/// # Arguments
/// - `guild_id`: A reference to a string slice representing the ID of the Discord guild/server.
/// - `anime_id`: A reference to an integer representing the ID of the anime to be removed.
/// - `db_config`: A `DbConfig` instance containing the connection configuration for the database.
///
/// # Returns
/// - `Result<()>`: Returns an `Ok(())` upon successful removal of the activity. Otherwise, returns an `Err` if any error occurs during the operation.
///
/// # Errors
/// This function may return an error if:
/// - The database connection fails.
/// - The query to find the activity data fails.
/// - The anime with the specified ID does not exist in the database (returns an error with a message).
/// - The deletion of the activity entry fails.
///
/// # Examples
/// ```rust
/// let guild_id = "123456789012345678";
/// let anime_id = 42;
/// let db_config = DbConfig::new("database_url");
///
/// match remove_activity(guild_id, &anime_id, db_config).await {
///     Ok(_) => println!("Activity successfully removed."),
///     Err(err) => eprintln!("Failed to remove activity: {}", err),
/// }
/// ```
async fn remove_activity(guild_id: &str, anime_id: &i32, db_config: DbConfig) -> Result<()> {
	let connection = sea_orm::Database::connect(get_url(db_config.clone())).await?;

	let activity = ActivityData::find()
		.filter(shared::database::activity_data::Column::ServerId.eq(guild_id))
		.filter(shared::database::activity_data::Column::AnimeId.eq(*anime_id))
		.one(&connection)
		.await?
		.ok_or(anyhow!(format!("Anime with id {} not found", anime_id)))?;

	activity.delete(&connection).await?;

	Ok(())
}
