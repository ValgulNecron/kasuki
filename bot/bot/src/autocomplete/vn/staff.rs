use crate::event_handler::BotData;
use crate::helper::get_option::subcommand::get_option_map_string_autocomplete_subcommand;
use serenity::all::{
	AutocompleteChoice, CommandInteraction, Context, CreateAutocompleteResponse,
	CreateInteractionResponse,
};
use shared::vndb::staff::get_staff;

pub async fn autocomplete(ctx: &Context, autocomplete_interaction: CommandInteraction) {
	let map = get_option_map_string_autocomplete_subcommand(&autocomplete_interaction);
	let bot_data = ctx.data::<BotData>().clone();

	let name = map.get("name").unwrap();

	let staff_root = get_staff(
		name.clone(),
		bot_data.vndb_cache.clone(),
		&bot_data.http_client,
	)
	.await
	.unwrap();

	let choices: Vec<AutocompleteChoice> = staff_root
		.results
		.iter()
		.take(25)
		.map(|s| AutocompleteChoice::new(s.name.clone(), s.id.clone()))
		.collect();

	let data = CreateAutocompleteResponse::new().set_choices(choices);
	let builder = CreateInteractionResponse::Autocomplete(data);

	if let Err(e) = autocomplete_interaction
		.create_response(&ctx.http, builder)
		.await
	{
		tracing::error!("Error sending response: {:?}", e);
	}
}
