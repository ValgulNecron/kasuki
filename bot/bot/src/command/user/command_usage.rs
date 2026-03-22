//! The `CommandUsageCommand` struct represents a specific implementation of a bot command in the
//! Discord bot framework using the `serenity` library. This command is used to display the usage
//! statistics of commands for a specific user.
//!
//! The struct contains the following fields:
//! - `ctx`: A `SerenityContext` instance that represents the context in which the bot operates.
//! - `command_interaction`: A `CommandInteraction` instance that represents the interaction data
//!   received from the user.
//!
//! Implements the `Command` trait which defines behavior for executing or interacting with a bot command.
//!
//! Example usage:
//! ```
//! let command_usage = CommandUsageCommand {
//!     ctx: serenity_context,
//!     command_interaction: command_interaction_data
//! };
//! let embed_contents = command_usage.get_contents().await?;
//! ```
use crate::command::context::CommandContext;
use crate::command::embed_content::{EmbedContent, EmbedsContents};
use crate::command::user::avatar::get_user_command;
use anyhow::Result;
use kasuki_macros::slash_command;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use shared::fluent_args;
use shared::localization::{Loader, USABLE_LOCALES};

#[slash_command(
	name = "command_usage", desc = "Show the usage of each command for an user.",
	command_type = SubCommand(parent = "user"),
	contexts = [Guild, BotDm, PrivateChannel],
	install_contexts = [Guild, User],
	args = [(name = "username", desc = "Username of the user you want the usage of.", arg_type = User, required = false, autocomplete = false)],
)]
async fn command_usage_command(self_: CommandUsageCommand) -> Result<EmbedsContents<'_>> {
	let cx = CommandContext::new(
		self_.get_ctx().clone(),
		self_.get_command_interaction().clone(),
	);
	let user = get_user_command(&cx.ctx, &cx.command_interaction).await?;

	let user_id = user.id.to_string();
	let username = user.name.clone();

	// Query database for user's command usage
	let usage = get_usage_for_id(&user_id, &*cx.db).await?;

	let lang_id = cx.lang_id().await;

	let mut embed_contents = vec![];

	let title_args = fluent_args!("user" => username.to_string());
	let embed_content = EmbedContent::new(USABLE_LOCALES.lookup_with_args(
		&lang_id,
		"user_command_usage-title",
		&title_args,
	));

	if usage.is_empty() {
		let args = fluent_args!("user" => username.to_string());
		let inner_embed = embed_content.description(USABLE_LOCALES.lookup_with_args(
			&lang_id,
			"user_command_usage-no_usage",
			&args,
		));
		embed_contents.push(inner_embed);
	} else {
		let mut description = String::new();

		let mut inner_embed = embed_content.clone();

		for (command, usage_count) in &usage {
			let args = fluent_args!("command" => command.clone(), "usage" => usage_count.to_string());
			description.push_str(
				USABLE_LOCALES
					.lookup_with_args(&lang_id, "user_command_usage-command_usage", &args)
					.as_str(),
			);

			description.push('\n');

			if description.len() > 4096 {
				let desc = description.clone();
				embed_contents.push(inner_embed.clone().description(desc));

				description.clear();

				inner_embed = embed_content.clone();
			}
		}

		if !description.is_empty() {
			embed_contents.push(inner_embed.clone().description(description));
		}
	}

	let embed_contents = EmbedsContents::new(embed_contents);

	Ok(embed_contents)
}

/// Retrieves usage statistics for a specific user ID from the database.
///
/// # Arguments
///
/// * `target_id` - A `&str` representing the target user ID whose usage information needs to be retrieved.
/// * `db_connection` - Database connection to query the command_usage table
///
/// # Returns
///
/// A `Vec<(String, u128)>` where each element is a tuple containing:
/// - `String`: The name of the command related to the usage.
/// - `u128`: The usage count for that command.
///
/// # Example
///
/// ```rust
/// let target_id = "user123";
/// let user_usage = get_usage_for_id(target_id, &db_connection).await?;
///
/// for (command, usage) in user_usage {
///     println!("Command: {}, Usage: {}", command, usage);
/// }
/// ```
async fn get_usage_for_id(
	target_id: &str, db_connection: &sea_orm::DatabaseConnection,
) -> Result<Vec<(String, u128)>, anyhow::Error> {
	use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

	// Query all command usage records for this user
	let usage_results = shared::database::command_usage::Entity::find()
		.filter(shared::database::command_usage::Column::User.eq(target_id))
		.all(db_connection)
		.await?;

	// Group by command and count
	let mut usage: std::collections::HashMap<String, u128> = std::collections::HashMap::new();
	for record in usage_results {
		*usage.entry(record.command).or_insert(0) += 1;
	}

	// Convert to sorted vector (descending by usage count)
	let mut usage: Vec<(String, u128)> = usage.into_iter().collect();
	usage.sort_by(|a, b| b.1.cmp(&a.1));

	Ok(usage)
}
