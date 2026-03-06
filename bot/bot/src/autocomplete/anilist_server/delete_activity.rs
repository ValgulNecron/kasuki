use crate::constant::{AUTOCOMPLETE_COUNT_LIMIT, DEFAULT_STRING};
use crate::event_handler::BotData;
use crate::helper::get_option::subcommand_group::get_option_map_string_autocomplete_subcommand_group;
use sea_orm::ColumnTrait;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use serenity::all::{
	AutocompleteChoice, CommandInteraction, Context as SerenityContext, CreateAutocompleteResponse,
	CreateInteractionResponse,
};
use shared::database::activity_data::Column;
use shared::database::prelude::ActivityData;

pub async fn autocomplete(ctx: SerenityContext, autocomplete_interaction: CommandInteraction) {
	let map = get_option_map_string_autocomplete_subcommand_group(&autocomplete_interaction);
	let bot_data = ctx.data::<BotData>().clone();

	let activity_search = map
		.get(&String::from("anime_name"))
		.map(String::as_str)
		.unwrap_or(DEFAULT_STRING);

	let guild_id = match autocomplete_interaction.guild_id {
		Some(id) => id.to_string(),
		None => String::from("0"),
	};

	let connection = bot_data.db_connection.clone();

	let activities = match ActivityData::find()
		.filter(Column::ServerId.eq(&guild_id))
		.all(&*connection)
		.await
	{
		Ok(data) => data,
		Err(e) => {
			tracing::debug!(?e);

			return;
		},
	};

	let mut choices = Vec::new();

	if activity_search.is_empty() {
		for activity in activities.iter().take(AUTOCOMPLETE_COUNT_LIMIT as usize) {
			choices.push(AutocompleteChoice::new(
				activity.name.clone(),
				activity.anime_id.to_string(),
			));
		}
	} else {
		let activity: Vec<String> = activities
			.clone()
			.into_iter()
			.map(|activity| format!("{}${}", activity.name, activity.anime_id))
			.collect();

		let activity_refs: Vec<&str> = activity.iter().map(String::as_str).collect();

		// Use rust-fuzzy-search to find the top 5 matches
		let matches = rust_fuzzy_search::fuzzy_search_best_n(
			activity_search,
			&activity_refs,
			AUTOCOMPLETE_COUNT_LIMIT as usize,
		);

		for (activity, _) in matches {
			let parts: Vec<&str> = activity.split('$').collect();

			let id = parts[1].to_string();

			let name = parts[0].to_string();

			choices.push(AutocompleteChoice::new(name, id))
		}
	}

	let data = CreateAutocompleteResponse::new().set_choices(choices);

	let builder = CreateInteractionResponse::Autocomplete(data);

	let _ = autocomplete_interaction
		.create_response(&ctx.http, builder)
		.await;
}
