use crate::autocomplete::anilist_server::{add_anime_activity, delete_activity};
use crate::autocomplete::anilist_user::{
	anime, character, compare, ln, manga, search, staff, studio, user,
};
use crate::autocomplete::game::steam_game_info;
use crate::autocomplete::management::give_premium_sub::give_premium_sub_autocomplete;
use crate::autocomplete::vn;
use crate::autocomplete::vn::{game, producer};
use crate::helper::get_option::subcommand_group::get_subcommand;
use serenity::all::{CommandInteraction, Context};
use tracing::trace;

pub async fn autocomplete_dispatching(ctx: Context, autocomplete_interaction: CommandInteraction) {
	trace!(?autocomplete_interaction);
	match autocomplete_interaction.data.name.as_str() {
		"admin" => admin_autocomplete(ctx, autocomplete_interaction).await,
		"anime" => anime::autocomplete(ctx, autocomplete_interaction).await,
		"ln" => ln::autocomplete(ctx, autocomplete_interaction).await,
		"manga" => manga::autocomplete(ctx, autocomplete_interaction).await,
		"user" => user::autocomplete(ctx, autocomplete_interaction).await,
		"character" => character::autocomplete(ctx, autocomplete_interaction).await,
		"compare" => compare::autocomplete(ctx, autocomplete_interaction).await,
		"register" => user::autocomplete(ctx, autocomplete_interaction).await,
		"staff" => staff::autocomplete(ctx, autocomplete_interaction).await,
		"studio" => studio::autocomplete(ctx, autocomplete_interaction).await,
		"search" => search::autocomplete(ctx, autocomplete_interaction).await,
		"seiyuu" => staff::autocomplete(ctx, autocomplete_interaction).await,
		"steam" => steam_autocomplete(ctx, autocomplete_interaction).await,
		"vn" => vn_autocomplete(ctx, autocomplete_interaction).await,
		"give_premium_sub" => give_premium_sub_autocomplete(ctx, autocomplete_interaction).await,
		_ => {},
	}
}

async fn admin_autocomplete(ctx: Context, autocomplete_interaction: CommandInteraction) {
	if autocomplete_interaction
		.data
		.options
		.first()
		.unwrap()
		.name
		.as_str()
		== "anilist"
	{
		anilist_admin_autocomplete(ctx, autocomplete_interaction).await
	}
}

async fn vn_autocomplete(ctx: Context, autocomplete_interaction: CommandInteraction) {
	match autocomplete_interaction
		.data
		.options
		.first()
		.unwrap()
		.name
		.as_str()
	{
		"game" => game::autocomplete(ctx, autocomplete_interaction).await,
		"character" => vn::character::autocomplete(ctx, autocomplete_interaction).await,
		"producer" => producer::autocomplete(ctx, autocomplete_interaction).await,
		_ => {},
	}
}

async fn anilist_admin_autocomplete(ctx: Context, autocomplete_interaction: CommandInteraction) {
	let interaction = autocomplete_interaction.clone();
	let subcommand = get_subcommand(&interaction).unwrap();

	let subcommand_name = subcommand.name;

	match subcommand_name {
		"add_anime_activity" => {
			add_anime_activity::autocomplete(ctx, autocomplete_interaction).await
		},
		"delete_activity" => delete_activity::autocomplete(ctx, autocomplete_interaction).await,
		_ => {},
	}
}

async fn steam_autocomplete(ctx: Context, autocomplete_interaction: CommandInteraction) {
	if autocomplete_interaction
		.data
		.options
		.first()
		.unwrap()
		.name
		.as_str()
		== "game"
	{
		steam_game_info::autocomplete(ctx, autocomplete_interaction).await
	}
}
