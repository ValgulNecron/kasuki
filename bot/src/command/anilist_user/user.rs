use crate::command::command_trait::{Command, Embed, SlashCommand};
use crate::database::prelude::RegisteredUser;
use crate::database::registered_user::Column;
use crate::event_handler::BotData;
use crate::get_url;
use crate::helper::get_option::command::get_option_map_string;
use crate::helper::make_graphql_cached::make_request_anilist;
use crate::structure::run::anilist::user;
use crate::structure::run::anilist::user::{
	User, UserQueryId, UserQueryIdVariables, UserQuerySearch, UserQuerySearchVariables,
};
use anyhow::{Result, anyhow};
use cynic::{GraphQlResponse, QueryBuilder};
use moka::future::Cache;
use sea_orm::ColumnTrait;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use small_fixed_array::FixedString;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct UserCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl Command for UserCommand {
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}
}

impl SlashCommand for UserCommand {
	async fn run_slash(&self) -> Result<()> {
		let ctx = self.get_ctx();
		let bot_data = ctx.data::<BotData>().clone();
		let command_interaction = &self.command_interaction;

		let config = bot_data.config.clone();

		let anilist_cache = bot_data.anilist_cache.clone();

		let map = get_option_map_string(command_interaction);

		let user = map.get(&FixedString::from_str_trunc("username"));

		// If the username is provided, fetch the user's data from AniList and send it as a response
		if let Some(value) = user {
			let data: User = get_user(value, anilist_cache.clone()).await?;

			let content =
				user::user_content(ctx, command_interaction, data, config.db.clone()).await?;

			self.send_embed(content).await?;
		}

		let user_id = &command_interaction.user.id.to_string();

		let connection = sea_orm::Database::connect(get_url(config.db.clone())).await?;

		let row = RegisteredUser::find()
			.filter(Column::UserId.eq(user_id))
			.one(&connection)
			.await?;

		let user = row.ok_or(anyhow!("No user found"))?;

		// Fetch the user's data from AniList and send it as a response
		let data = get_user(user.anilist_id.to_string().as_str(), anilist_cache).await?;

		let content = user::user_content(ctx, command_interaction, data, config.db.clone()).await?;

		self.send_embed(content).await
	}
}

pub async fn get_user(
	value: &str, anilist_cache: Arc<RwLock<Cache<String, String>>>,
) -> Result<User> {
	// If the value is a valid user ID, fetch the user's data by ID
	let user = if value.parse::<i32>().is_ok() {
		let id = value.parse::<i32>()?;

		let var = UserQueryIdVariables { id: Some(id) };

		let operation = UserQueryId::build(var);

		let data: GraphQlResponse<UserQueryId> =
			make_request_anilist(operation, false, anilist_cache).await?;

		data.data.unwrap().user.unwrap()
	} else {
		// If the value is not a valid user ID, fetch the user's data by username
		let var = UserQuerySearchVariables {
			search: Some(value),
		};

		let operation = UserQuerySearch::build(var);

		let data: GraphQlResponse<UserQuerySearch> =
			make_request_anilist(operation, false, anilist_cache).await?;

		data.data.unwrap().user.unwrap()
	};

	Ok(user)
}
