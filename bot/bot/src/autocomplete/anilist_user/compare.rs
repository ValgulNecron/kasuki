use serenity::all::{
	AutocompleteChoice, CommandInteraction, Context, CreateAutocompleteResponse,
	CreateInteractionResponse,
};

use crate::event_handler::BotData;
use shared::anilist::autocomplete::search_users;

pub async fn autocomplete(ctx: &Context, autocomplete_interaction: CommandInteraction) {
	let bot_data = ctx.data::<BotData>().clone();

	let focused = match autocomplete_interaction.data.autocomplete() {
		Some(opt) => opt,
		None => return,
	};

	let choices = get_choices(focused.value, &bot_data).await;

	let data = CreateAutocompleteResponse::new().set_choices(choices);

	let builder = CreateInteractionResponse::Autocomplete(data);

	let _ = autocomplete_interaction
		.create_response(&ctx.http, builder)
		.await;
}

async fn get_choices<'a>(search: &'a str, bot_data: &'a BotData) -> Vec<AutocompleteChoice<'a>> {
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
