use cynic::{GraphQlResponse, QueryBuilder};
use serenity::all::{
	AutocompleteChoice, CommandInteraction, Context, CreateAutocompleteResponse,
	CreateInteractionResponse,
};

use crate::constant::DEFAULT_STRING;
use crate::event_handler::BotData;
use crate::helper::get_option::subcommand::get_option_map_string_autocomplete_subcommand;
use crate::helper::make_graphql_cached::make_request_anilist;
use crate::structure::autocomplete::anilist::staff::{
	StaffAutocomplete, StaffAutocompleteVariables,
};

pub async fn autocomplete(ctx: Context, autocomplete_interaction: CommandInteraction) {
	let map = get_option_map_string_autocomplete_subcommand(&autocomplete_interaction);
	let bot_data = ctx.data::<BotData>().clone();

	let staff_search = map
		.get(&String::from("staff_name"))
		.unwrap_or(DEFAULT_STRING);

	let var = StaffAutocompleteVariables {
		search: Some(staff_search),
	};

	let operation = StaffAutocomplete::build(var);

	let data: GraphQlResponse<StaffAutocomplete> =
		match make_request_anilist(operation, false, bot_data.anilist_cache.clone()).await {
			Ok(data) => data,
			Err(e) => {
				tracing::error!(?e);

				return;
			},
		};

	let mut choices = Vec::new();

	let staffs = match data.data {
		Some(data) => match data.page {
			Some(page) => match page.staff {
				Some(staff) => staff,
				None => {
					return;
				},
			},
			None => {
				return;
			},
		},
		None => return,
	};

	for staff in staffs {
		let data = staff.unwrap();

		let name = data.name.unwrap();

		let user_pref = name.user_preferred;

		let native = name.native;

		let full = name.full;

		let name = user_pref.unwrap_or(native.unwrap_or(full.unwrap_or(DEFAULT_STRING.clone())));

		choices.push(AutocompleteChoice::new(name, data.id.to_string()))
	}

	let data = CreateAutocompleteResponse::new().set_choices(choices);

	let builder = CreateInteractionResponse::Autocomplete(data);

	let _ = autocomplete_interaction
		.create_response(&ctx.http, builder)
		.await;
}
