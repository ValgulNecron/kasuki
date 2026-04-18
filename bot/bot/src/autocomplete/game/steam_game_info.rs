use serenity::all::{
	AutocompleteChoice, CommandInteraction, Context, CreateAutocompleteResponse,
	CreateInteractionResponse,
};
use tracing::{debug, trace};

use crate::constant::{AUTOCOMPLETE_COUNT_LIMIT, DEFAULT_STRING};
use crate::event_handler::BotData;
use crate::helper::get_option::subcommand::get_option_map_string_autocomplete_subcommand;

pub async fn autocomplete(ctx: &Context, autocomplete_interaction: CommandInteraction) {
	let map = get_option_map_string_autocomplete_subcommand(&autocomplete_interaction);
	let bot_data = ctx.data::<BotData>();

	let game_search = map
		.get("game_name")
		.map(String::as_str)
		.unwrap_or(DEFAULT_STRING);

	trace!("game_search: {}", game_search);

	if game_search.is_empty() {
		let data = CreateAutocompleteResponse::new().set_choices(Vec::<AutocompleteChoice>::new());
		let builder = CreateInteractionResponse::Autocomplete(data);
		let _ = autocomplete_interaction
			.create_response(&ctx.http, builder)
			.await;
		return;
	}

	let index = bot_data.apps.load();
	let results = index.search(game_search, AUTOCOMPLETE_COUNT_LIMIT as usize);

	let choices: Vec<AutocompleteChoice> = results
		.into_iter()
		.map(|(name, app_id)| {
			let name_show = if name.len() > 100 {
				name.chars().take(100).collect::<String>()
			} else {
				name.to_string()
			};
			AutocompleteChoice::new(name_show, app_id.to_string())
		})
		.collect();

	let data = CreateAutocompleteResponse::new().set_choices(choices);
	let builder = CreateInteractionResponse::Autocomplete(data);

	if let Err(why) = autocomplete_interaction
		.create_response(&ctx.http, builder)
		.await
	{
		debug!("Error sending response: {:?}", why);
	}
}
