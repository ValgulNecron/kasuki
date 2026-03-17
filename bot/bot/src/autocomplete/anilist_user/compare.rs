use serenity::all::{
	AutocompleteChoice, CommandInteraction, Context, CreateAutocompleteResponse,
	CreateInteractionResponse,
};
use small_fixed_array::FixedString;
use tracing::trace;

use crate::constant::DEFAULT_STRING;
use crate::event_handler::BotData;
use crate::helper::get_option::command::get_option_map_string;
use shared::anilist::autocomplete::search_users;

pub async fn autocomplete(ctx: &Context, autocomplete_interaction: CommandInteraction) {
	let mut choices = Vec::new();
	let bot_data = ctx.data::<BotData>().clone();

	trace!("{:?}", &autocomplete_interaction.data.options);

	let map = get_option_map_string(&autocomplete_interaction);

	let user1 = map
		.get(&FixedString::from_str_trunc("username"))
		.map(String::as_str)
		.unwrap_or(DEFAULT_STRING);

	choices.extend(get_choices(user1, &bot_data).await);

	let user2 = map
		.get(&FixedString::from_str_trunc("username2"))
		.map(String::as_str)
		.unwrap_or(DEFAULT_STRING);

	choices.extend(get_choices(user2, &bot_data).await);

	let data = CreateAutocompleteResponse::new().set_choices(choices);

	let builder = CreateInteractionResponse::Autocomplete(data);

	let _ = autocomplete_interaction
		.create_response(&ctx.http, builder)
		.await;
}

async fn get_choices<'a>(search: &'a str, bot_data: &'a BotData) -> Vec<AutocompleteChoice<'a>> {
	trace!("{:?}", search);

	match search_users(search, bot_data.anilist_cache.clone()).await {
		Ok(results) => results
			.into_iter()
			.map(|(name, id)| AutocompleteChoice::new(name, id))
			.collect(),
		Err(e) => {
			tracing::error!(?e);

			Vec::new()
		},
	}
}
