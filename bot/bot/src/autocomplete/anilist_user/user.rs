use serenity::all::{
	AutocompleteChoice, CommandInteraction, Context, CreateAutocompleteResponse,
	CreateInteractionResponse,
};
use small_fixed_array::FixedString;

use crate::constant::DEFAULT_STRING;
use crate::event_handler::BotData;
use crate::helper::get_option::command::get_option_map_string;
use shared::anilist::autocomplete::search_users;

pub async fn autocomplete(ctx: &Context, autocomplete_interaction: CommandInteraction) {
	let map = get_option_map_string(&autocomplete_interaction);
	let bot_data = ctx.data::<BotData>().clone();

	let user_search = map
		.get(&FixedString::from_str_trunc("username"))
		.map(String::as_str)
		.unwrap_or(DEFAULT_STRING);

	let results = match search_users(user_search, bot_data.anilist_cache.clone()).await {
		Ok(results) => results,
		Err(e) => {
			tracing::error!(?e);

			return;
		},
	};

	let choices: Vec<AutocompleteChoice> = results
		.into_iter()
		.map(|(name, id)| AutocompleteChoice::new(name, id))
		.collect();

	let data = CreateAutocompleteResponse::new().set_choices(choices);

	let builder = CreateInteractionResponse::Autocomplete(data);

	let _ = autocomplete_interaction
		.create_response(&ctx.http, builder)
		.await;
}
