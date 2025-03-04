use crate::event_handler::BotData;
use crate::helper::get_option::subcommand::get_option_map_string_autocomplete_subcommand;
use crate::helper::vndbapi::producer::{Producer, get_producer};
use serenity::all::{
	AutocompleteChoice, CommandInteraction, Context, CreateAutocompleteResponse,
	CreateInteractionResponse,
};
use tracing::trace;

pub async fn autocomplete(ctx: Context, autocomplete_interaction: CommandInteraction) {
	let map = get_option_map_string_autocomplete_subcommand(&autocomplete_interaction);
	let bot_data = ctx.data::<BotData>().clone();

	let game = map.get(&String::from("name")).unwrap();

	let producer = get_producer(game.clone(), bot_data.vndb_cache.clone())
		.await
		.unwrap();

	let producer = producer.results;

	// take the 25 first results
	let vn_result: Vec<Producer> = producer.iter().take(25).cloned().collect();

	let mut choices = Vec::new();

	trace!("Game: {}", game);

	trace!("Map: {:?}", map);

	for vn in vn_result {
		choices.push(AutocompleteChoice::new(vn.name.clone(), vn.id.clone()))
	}

	let data = CreateAutocompleteResponse::new().set_choices(choices);

	let builder = CreateInteractionResponse::Autocomplete(data);

	let _ = autocomplete_interaction
		.create_response(&ctx.http, builder)
		.await;
}
