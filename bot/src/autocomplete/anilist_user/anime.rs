use serenity::all::{CommandInteraction, Context};
use small_fixed_array::FixedString;
use tracing::trace;

use crate::constant::DEFAULT_STRING;
use crate::event_handler::BotData;
use crate::helper::get_option::command::get_option_map_string;
use crate::structure::autocomplete::anilist::media::{
	MediaAutocompleteVariables, MediaFormat, MediaType, send_auto_complete,
};

pub async fn autocomplete(ctx: Context, autocomplete_interaction: CommandInteraction) {
	let map = get_option_map_string(&autocomplete_interaction);
	let bot_data = ctx.data::<BotData>().clone();
	let anime_search = map
		.get(&FixedString::from_str_trunc("anime_name"))
		.unwrap_or(DEFAULT_STRING);

	trace!("anime_search: {}", anime_search);

	let var = get_autocomplete_media_variables(anime_search);

	send_auto_complete(
		&ctx,
		autocomplete_interaction,
		var,
		bot_data.anilist_cache.clone(),
	)
	.await;
}

pub fn get_autocomplete_media_variables(anime_search: &str) -> MediaAutocompleteVariables {
	MediaAutocompleteVariables {
		search: Some(anime_search),
		in_media_format: Some(vec![
			Some(MediaFormat::Tv),
			Some(MediaFormat::TvShort),
			Some(MediaFormat::Movie),
			Some(MediaFormat::Special),
			Some(MediaFormat::Ova),
			Some(MediaFormat::Ona),
			Some(MediaFormat::Music),
		]),
		media_type: Some(MediaType::Anime),
	}
}
