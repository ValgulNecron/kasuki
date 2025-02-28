use crate::constant::DEFAULT_STRING;
use crate::event_handler::BotData;
use crate::helper::get_option::command::get_option_map_string;
use crate::structure::autocomplete::anilist::media::{
	MediaAutocompleteVariables, MediaFormat, MediaType, send_auto_complete,
};
use serenity::all::{CommandInteraction, Context};
use small_fixed_array::FixedString;

pub async fn autocomplete(ctx: Context, autocomplete_interaction: CommandInteraction) {
	let map = get_option_map_string(&autocomplete_interaction);
	let bot_data = ctx.data::<BotData>().clone();

	let manga_search = map
		.get(&FixedString::from_str_trunc("manga_name"))
		.unwrap_or(DEFAULT_STRING);

	let var = MediaAutocompleteVariables {
		search: Some(manga_search.as_str()),
		in_media_format: Some(vec![Some(MediaFormat::Manga), Some(MediaFormat::OneShot)]),
		media_type: Some(MediaType::Manga),
	};

	send_auto_complete(
		&ctx,
		autocomplete_interaction,
		var,
		bot_data.anilist_cache.clone(),
	)
	.await;
}
