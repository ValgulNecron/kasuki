use anyhow::{anyhow, Result};
use std::sync::Arc;

use moka::future::Cache;
use sea_orm::ActiveValue::Set;
use sea_orm::EntityTrait;
use serenity::all::{
	CommandInteraction, Context as SerenityContext, CreateInteractionResponse,
	CreateInteractionResponseMessage,
};
use small_fixed_array::FixedString;
use tokio::sync::RwLock;

use crate::command::anilist_user::user::get_user;
use crate::command::command_trait::{Command, Embed, EmbedContent, EmbedType, SlashCommand};
use crate::config::Config;
use crate::database::prelude::RegisteredUser;
use crate::database::registered_user::{ActiveModel, Column};
use crate::event_handler::BotData;
use crate::get_url;
use crate::helper::create_default_embed::get_default_embed;
use crate::helper::get_option::command::get_option_map_string;
use crate::structure::message::anilist_user::register::load_localization_register;
use crate::structure::run::anilist::user::{get_color, get_user_url, User};

pub struct RegisterCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl Command for RegisterCommand {
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}
}

impl SlashCommand for RegisterCommand {
	async fn run_slash(&self) -> Result<()> {
		let ctx = self.get_ctx();
		let bot_data = ctx.data::<BotData>().clone();
		let command_interaction = self.get_command_interaction();

		let anilist_cache = bot_data.anilist_cache.clone();
		let config = bot_data.config.clone();

		let map = get_option_map_string(command_interaction);

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

		// Load the localized register strings
		let register_localised = load_localization_register(guild_id, config.db.clone()).await?;

		// Retrieve the user's Discord ID and username
		let user_id = &command_interaction.user.id.to_string();

		let username = &command_interaction.user.name;

		// Register the user's AniList account by storing the user's Discord ID and AniList ID in the database
		let connection = sea_orm::Database::connect(get_url(config.db.clone())).await?;

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
			.exec(&connection)
			.await?;

		// Construct the description for the embed
		let desc = register_localised
			.desc
			.replace("$user$", username.as_str())
			.replace("$id$", user_id)
			.replace("$anilist$", user_data.name.clone().as_str());

		let content = EmbedContent {
			title: user_data.clone().name,
			description: desc,
			thumbnail: Some(user_data.clone().avatar.unwrap().large.unwrap()),
			url: Some(get_user_url(user_data.id)),
			command_type: EmbedType::First,
			colour: Some(get_color(user_data.clone())),
			fields: vec![],
			images: None,
			action_row: None,
			images_url: None,
		};

		self.send_embed(content).await
	}
}
