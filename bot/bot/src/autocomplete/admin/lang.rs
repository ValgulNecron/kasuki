use serenity::all::{
	AutocompleteChoice, CommandInteraction, Context, CreateAutocompleteResponse,
	CreateInteractionResponse,
};
use shared::localization::available_locales;
use tracing::debug;

pub async fn autocomplete(ctx: &Context, autocomplete_interaction: CommandInteraction) {
	let choices: Vec<AutocompleteChoice> = available_locales()
		.into_iter()
		.map(|locale| AutocompleteChoice::new(locale.clone(), locale))
		.collect();

	let data = CreateAutocompleteResponse::new().set_choices(choices);
	let builder = CreateInteractionResponse::Autocomplete(data);

	if let Err(why) = autocomplete_interaction
		.create_response(&ctx.http, builder)
		.await
	{
		debug!("Error sending lang autocomplete response: {:?}", why);
	}
}
