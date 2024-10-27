use cynic::{GraphQlResponse, QueryBuilder};
use serenity::all::{
	AutocompleteChoice, CommandInteraction, Context, CreateAutocompleteResponse,
	CreateInteractionResponse,
};

use crate::constant::DEFAULT_STRING;
use crate::event_handler::BotData;
use crate::helper::get_option::subcommand::get_option_map_string_autocomplete_subcommand;
use crate::helper::make_graphql_cached::make_request_anilist;
use crate::structure::autocomplete::anilist::user::{UserAutocomplete, UserAutocompleteVariables};

pub async fn autocomplete(ctx: Context, autocomplete_interaction: CommandInteraction) {
	let map = get_option_map_string_autocomplete_subcommand(&autocomplete_interaction);
	let bot_data = ctx.data::<BotData>().clone();

	let user_search = map.get(&String::from("username")).unwrap_or(DEFAULT_STRING);

	let var = UserAutocompleteVariables {
		search: Some(user_search),
	};

	let operation = UserAutocomplete::build(var);

	let data: GraphQlResponse<UserAutocomplete> =
		match make_request_anilist(operation, false, bot_data.anilist_cache.clone()).await {
			Ok(data) => data,
			Err(e) => {
				tracing::error!(?e);

				return;
			},
		};

	let users = match data.data {
		Some(data) => match data.page {
			Some(page) => match page.users {
				Some(users) => users,
				None => return,
			},
			None => return,
		},
		None => {
			tracing::debug!(?data.errors);

			return;
		},
	};

	let mut choices = Vec::new();

	for user in users {
		let data = user.unwrap();

		let user = data.name;

		choices.push(AutocompleteChoice::new(user, data.id.to_string()))
	}

	let data = CreateAutocompleteResponse::new().set_choices(choices);

	let builder = CreateInteractionResponse::Autocomplete(data);

	let _ = autocomplete_interaction
		.create_response(&ctx.http, builder)
		.await;
}
