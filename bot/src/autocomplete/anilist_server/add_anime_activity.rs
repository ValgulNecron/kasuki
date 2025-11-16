use serenity::all::{CommandInteraction, Context};
use tracing::trace;

use crate::autocomplete::anilist_user::anime::get_autocomplete_media_variables;
use crate::constant::DEFAULT_STRING;
use crate::event_handler::BotData;
use crate::helper::get_option::subcommand_group::get_option_map_string_autocomplete_subcommand_group;
use crate::structure::autocomplete::anilist::media::send_auto_complete;

pub async fn autocomplete(ctx: Context, autocomplete_interaction: CommandInteraction) {
	let map = get_option_map_string_autocomplete_subcommand_group(&autocomplete_interaction);
	let bot_data = ctx.data::<BotData>().clone();
	trace!("{:?}", map);

	let anime_search = map
		.get(&String::from("anime_name"))
		.unwrap_or(DEFAULT_STRING);

	let var = get_autocomplete_media_variables(anime_search);

	send_auto_complete(
		&ctx,
		autocomplete_interaction,
		var,
		bot_data.anilist_cache.clone(),
	)
	.await;
}
