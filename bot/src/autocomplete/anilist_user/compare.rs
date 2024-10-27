use std::sync::Arc;

use cynic::{GraphQlResponse, QueryBuilder};
use moka::future::Cache;
use serenity::all::{
	AutocompleteChoice, CommandInteraction, Context as SerenityContext, CreateAutocompleteResponse,
	CreateInteractionResponse,
};
use tokio::sync::RwLock;
use tracing::log::trace;

use crate::constant::DEFAULT_STRING;
use crate::event_handler::BotData;
use crate::helper::get_option::subcommand::get_option_map_string_autocomplete_subcommand;
use crate::helper::make_graphql_cached::make_request_anilist;
use crate::structure::autocomplete::anilist::user::{UserAutocomplete, UserAutocompleteVariables};
use anyhow::Result;

pub async fn autocomplete(ctx: SerenityContext, autocomplete_interaction: CommandInteraction) {
	let mut choice = Vec::new();
	let bot_data = ctx.data::<BotData>().clone();

	trace!("{:?}", &autocomplete_interaction.data.options);

	let map = get_option_map_string_autocomplete_subcommand(&autocomplete_interaction);

	let user1 = map.get(&String::from("username")).unwrap_or(DEFAULT_STRING);

	choice.extend(get_choices(user1, bot_data.anilist_cache.clone()).await);

	let user2 = map
		.get(&String::from("username2"))
		.unwrap_or(DEFAULT_STRING);

	choice.extend(get_choices(user2, bot_data.anilist_cache.clone()).await);

	let data = CreateAutocompleteResponse::new().set_choices(choice);

	let builder = CreateInteractionResponse::Autocomplete(data);

	let _ = autocomplete_interaction
		.create_response(&ctx.http, builder)
		.await;
}

async fn get_choices(
	search: &str, anilist_cache: Arc<RwLock<Cache<String, String>>>,
) -> Vec<AutocompleteChoice> {
	trace!("{:?}", search);

	let var = UserAutocompleteVariables {
		search: Some(search),
	};

	let operation = UserAutocomplete::build(var);

	let data: Result<GraphQlResponse<UserAutocomplete>> =
		make_request_anilist(operation, false, anilist_cache).await;

	let data = match data {
		Ok(data) => data,
		Err(e) => {
			tracing::error!(?e);

			return Vec::new();
		},
	};

	let users = match data.data {
		Some(data) => match data.page {
			Some(page) => match page.users {
				Some(users) => users,
				None => return Vec::new(),
			},
			None => return Vec::new(),
		},
		None => {
			tracing::error!(?data.errors);

			return Vec::new();
		},
	};

	let mut choices = Vec::new();

	for user in users {
		let data = user.unwrap();

		let user = data.name;

		choices.push(AutocompleteChoice::new(user, data.id.to_string()))
	}

	choices
}
