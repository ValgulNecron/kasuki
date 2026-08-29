use serenity::all::{
	AutocompleteChoice, CommandInteraction, Context, CreateAutocompleteResponse,
	CreateInteractionResponse,
};
use small_fixed_array::FixedString;
use tracing::trace;

use crate::constant::DEFAULT_STRING;
use crate::event_handler::BotData;
use crate::helper::get_option::command::get_option_map_string;
use shared::anilist::autocomplete::media::{MediaFormat, MediaType};
use shared::anilist::autocomplete::search_media;

pub async fn autocomplete(ctx: &Context, autocomplete_interaction: CommandInteraction) {
	let map = get_option_map_string(&autocomplete_interaction);
	let bot_data = ctx.data::<BotData>().clone();

	let anime_search = map
		.get(&FixedString::from_str_trunc("anime_name"))
		.map(String::as_str)
		.unwrap_or(DEFAULT_STRING);

	trace!("anime_search: {}", anime_search);

	let anime_formats = vec![
		Some(MediaFormat::Tv),
		Some(MediaFormat::TvShort),
		Some(MediaFormat::Movie),
		Some(MediaFormat::Special),
		Some(MediaFormat::Ova),
		Some(MediaFormat::Ona),
		Some(MediaFormat::Music),
	];

	let results = match search_media(
		anime_search,
		Some(MediaType::Anime),
		Some(anime_formats),
		bot_data.anilist_cache.clone(),
	)
	.await
	{
		Ok(results) => results,
		Err(e) => {
			tracing::debug!(?e);

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
