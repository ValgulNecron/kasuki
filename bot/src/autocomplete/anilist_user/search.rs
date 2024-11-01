use crate::autocomplete::anilist_user::{anime, character, ln, manga, staff, studio, user};
use crate::constant::DEFAULT_STRING;
use crate::helper::get_option::command::get_option_map_string;
use crate::helper::get_option::subcommand::get_option_map_string_autocomplete_subcommand;
use serenity::all::{CommandInteraction, Context};
use small_fixed_array::FixedString;

pub async fn autocomplete(ctx: Context, autocomplete_interaction: CommandInteraction) {
	let map = get_option_map_string(&autocomplete_interaction);

	let search_type = map
		.get(&FixedString::from_str_trunc("type"))
		.unwrap_or(DEFAULT_STRING);

	match search_type.as_str() {
		"anime" => anime::autocomplete(ctx, autocomplete_interaction).await,
		"ln" => ln::autocomplete(ctx, autocomplete_interaction).await,
		"manga" => manga::autocomplete(ctx, autocomplete_interaction).await,
		"user" => user::autocomplete(ctx, autocomplete_interaction).await,
		"character" => character::autocomplete(ctx, autocomplete_interaction).await,
		"staff" => staff::autocomplete(ctx, autocomplete_interaction).await,
		"studio" => studio::autocomplete(ctx, autocomplete_interaction).await,
		_ => {},
	}
}
