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

use sea_orm::ActiveValue::Set;
use sea_orm::EntityTrait;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use small_fixed_array::FixedString;

use crate::command::anilist_user::user::get_user;
use crate::command::command::{Command, CommandRun};
use crate::command::embed_content::{CommandType, EmbedContent, EmbedsContents};
use crate::database::prelude::RegisteredUser;
use crate::database::registered_user::{ActiveModel, Column};
use crate::event_handler::BotData;
use crate::helper::get_option::command::get_option_map_string;
use crate::impl_command;
use crate::structure::message::anilist_user::register::load_localization_register;
use crate::structure::run::anilist::user::{get_color, get_user_url, User};

#[derive(Clone)]
pub struct RegisterCommand {
    pub ctx: SerenityContext,
    pub command_interaction: CommandInteraction,
}

impl_command!(
	for RegisterCommand,
	get_contents = |self_: RegisterCommand| async move {
		let ctx = self_.get_ctx().clone();
		let bot_data = ctx.data::<BotData>().clone();
		let command_interaction = self_.get_command_interaction().clone();

	let anilist_cache = bot_data.anilist_cache.clone();
		let connection = bot_data.db_connection.clone();

		let map = get_option_map_string(&command_interaction);

		self_.defer().await?;

		let value = map
			.get(&FixedString::from_str_trunc("username"))
			.ok_or(anyhow!("No username provided"))?;

		// Fetch the user data from AniList
		let user_data: User = get_user(value, anilist_cache).await?;

		// Retrieve the guild ID from the command interaction
		let guild_id = match command_interaction.guild_id {
			Some(id) => id.to_string(),
			None => String::from("0"),
		};
		let db_connection = bot_data.db_connection.clone();

		// Load the localized register strings
		let register_localised = load_localization_register(guild_id, db_connection).await?;

		// Retrieve the user's Discord ID and username
		let user_id = &command_interaction.user.id.to_string();

		let username = &command_interaction.user.name;

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

		// Construct the description for the embed
		let desc = register_localised
			.desc
			.replace("$user$", username.as_str())
			.replace("$id$", user_id)
			.replace("$anilist$", user_data.name.clone().as_str());

		let embed_content = EmbedContent::new(user_data.clone().name)
			.description(desc)
			.thumbnail(user_data.clone().avatar.unwrap().large.unwrap())
			.url(get_user_url(&user_data.id))
			.colour(get_color(user_data.clone()));

		let embed_contents = EmbedsContents::new(CommandType::Followup, vec![embed_content]);

		Ok(embed_contents)
	}
);
