//! The `RegisterCommand` struct is responsible for handling the "register" command interaction
//! within a Discord bot. It is part of a bot infrastructure that integrates with the AniList API
//! and a database for user registrations.
//!
//! This command allows users to register their AniList account with the bot, storing a mapping
//! between their Discord ID and AniList ID in the database.
//!
//! # Fields
//! - `ctx`: The `SerenityContext` that represents the current bot state and provides access to shared
//!          data like the database connection, configuration, etc.
//! - `command_interaction`: The command interaction event received from Discord, containing details
//!                          about the invoked command (i.e., the user, arguments, and guild information).
//!
//! This struct implements the `Command` trait, defining the behavior and response of the "register" command.
use anyhow::anyhow;

use fluent_templates::fluent_bundle::FluentValue;
use fluent_templates::Loader;
use kasuki_macros::slash_command;
use sea_orm::ActiveValue::Set;
use sea_orm::EntityTrait;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use shared::localization::USABLE_LOCALES;
use small_fixed_array::FixedString;
use std::borrow::Cow;
use std::collections::HashMap;

use crate::command::context::CommandContext;
use crate::command::embed_content::{EmbedContent, EmbedsContents};
use crate::helper::get_option::command::get_option_map_string;
use crate::structure::run::anilist::user::{get_color, get_user, get_user_url, User};
use shared::database::prelude::RegisteredUser;
use shared::database::registered_user::{ActiveModel, Column};

#[slash_command(
	name = "register", desc = "Register your username on AniList.", command_type = ChatInput,
	contexts = [Guild, BotDm, PrivateChannel],
	install_contexts = [Guild, User],
	args = [(name = "username", desc = "Username you want to register.", arg_type = String, required = true, autocomplete = true)],
)]
async fn register_command(self_: RegisterCommand) -> Result<EmbedsContents<'_>> {
	let cx = CommandContext::new(
		self_.get_ctx().clone(),
		self_.get_command_interaction().clone(),
	);
	let anilist_cache = cx.anilist_cache.clone();
	let connection = cx.db.clone();

	let map = get_option_map_string(&cx.command_interaction);

	let value = map
		.get(&FixedString::from_str_trunc("username"))
		.ok_or(anyhow!("No username provided"))?;

	// Fetch the user data from AniList
	let user_data: User = get_user(value, anilist_cache).await?;

	// Get the language identifier for localization
	let lang_id = cx.lang_id().await;

	// Retrieve the user's Discord ID and username
	let user_id = &cx.command_interaction.user.id.to_string();

	let username = &cx.command_interaction.user.name;

	RegisteredUser::insert(ActiveModel {
		user_id: Set(user_id.to_string()),
		anilist_id: Set(user_data.id),
		..Default::default()
	})
	.on_conflict(
		sea_orm::sea_query::OnConflict::column(Column::AnilistId)
			.update_column(Column::AnilistId)
			.to_owned(),
	)
	.exec(&*connection)
	.await?;

	// Construct the description for the embed using Fluent
	let mut args: HashMap<Cow<'static, str>, FluentValue<'_>> = HashMap::new();
	args.insert(Cow::Borrowed("user"), FluentValue::from(username.as_str()));
	args.insert(Cow::Borrowed("id"), FluentValue::from(user_id.as_str()));
	args.insert(
		Cow::Borrowed("anilist"),
		FluentValue::from(user_data.name.clone()),
	);

	let desc = USABLE_LOCALES.lookup_with_args(&lang_id, "anilist_user_register-desc", &args);

	let embed_content = EmbedContent::new(user_data.clone().name)
		.description(desc)
		.thumbnail(user_data.clone().avatar.unwrap().large.unwrap())
		.url(get_user_url(&user_data.id))
		.colour(get_color(user_data.clone()));

	let embed_contents = EmbedsContents::new(vec![embed_content]);

	Ok(embed_contents)
}
