use serenity::all::{
	AutocompleteChoice, CommandInteraction, Context, CreateAutocompleteResponse,
	CreateInteractionResponse,
};
use tracing::{debug, trace};

use crate::constant::{AUTOCOMPLETE_COUNT_LIMIT, DEFAULT_STRING};
use crate::event_handler::BotData;
use crate::helper::fuzzy_search::distance_top_n;
use crate::helper::get_option::subcommand::get_option_map_string_autocomplete_subcommand;

pub async fn autocomplete(ctx: Context, autocomplete_interaction: CommandInteraction) {
	let map = get_option_map_string_autocomplete_subcommand(&autocomplete_interaction);
	let bot_data = ctx.data::<BotData>().clone();

	let game_search = map
		.get(&String::from("game_name"))
		.map(String::as_str)
		.unwrap_or(DEFAULT_STRING);

	trace!("game_search: {}", game_search);

	// If search is empty, respond with no choices to avoid unnecessary work
	if game_search.is_empty() {
		let data = CreateAutocompleteResponse::new().set_choices(Vec::<AutocompleteChoice>::new());
		let builder = CreateInteractionResponse::Autocomplete(data);
		let _ = autocomplete_interaction
			.create_response(&ctx.http, builder)
			.await;
		return;
	}

	let search_lc = game_search.to_ascii_lowercase();

	let guard = bot_data.apps.read().await;

	// Build a limited candidate set with priority to prefix matches, then substring matches (case-insensitive)
	const MAX_CANDIDATES: usize = 500;
	let mut prefix_candidates: Vec<&str> = Vec::with_capacity(128);
	let mut contain_candidates: Vec<&str> = Vec::with_capacity(512);

	for name in guard.keys() {
		let name_str: &str = name.as_str();
		// Fast path: exact or prefix (case-insensitive, no allocation for prefix compare)
		if name_str.len() >= game_search.len()
			&& name_str[..game_search.len()].eq_ignore_ascii_case(game_search)
		{
			prefix_candidates.push(name_str);
			if prefix_candidates.len() >= MAX_CANDIDATES {
				break;
			}
			continue;
		}

		// Fallback: case-sensitive contains (cheap) then case-insensitive contains
		if name_str.contains(game_search) {
			contain_candidates.push(name_str);
		} else if name_str.to_ascii_lowercase().contains(&search_lc) {
			contain_candidates.push(name_str);
		}

		// Stop early if we already have enough candidates overall
		if prefix_candidates.len() + contain_candidates.len() >= MAX_CANDIDATES {
			break;
		}
	}

	// Merge candidates, keeping prefixes first and truncating to MAX_CANDIDATES
	let mut candidates: Vec<&str> = prefix_candidates;
	if candidates.len() < MAX_CANDIDATES {
		let remaining = MAX_CANDIDATES - candidates.len();
		candidates.extend(contain_candidates.into_iter().take(remaining));
	}

	// If we still have no candidates, respond with empty choices
	if candidates.is_empty() {
		let data = CreateAutocompleteResponse::new().set_choices(Vec::<AutocompleteChoice>::new());
		let builder = CreateInteractionResponse::Autocomplete(data);
		let _ = autocomplete_interaction
			.create_response(&ctx.http, builder)
			.await;
		return;
	}

	let game_search_owned = game_search.to_string();
	let owned_candidates: Vec<String> = candidates.into_iter().map(|s| s.to_string()).collect();
	drop(guard); // Release lock during CPU-heavy fuzzy search
	let result = tokio::task::spawn_blocking(move || {
		let refs: Vec<&str> = owned_candidates.iter().map(|s| s.as_str()).collect();
		distance_top_n(
			game_search_owned.as_str(),
			refs,
			AUTOCOMPLETE_COUNT_LIMIT as usize,
		)
	})
	.await;

	let result = match result {
		Ok(Ok(r)) => r,
		other => {
			debug!("Error in fuzzy ranking: {:?}", other);
			let data =
				CreateAutocompleteResponse::new().set_choices(Vec::<AutocompleteChoice>::new());
			let builder = CreateInteractionResponse::Autocomplete(data);
			let _ = autocomplete_interaction
				.create_response(&ctx.http, builder)
				.await;
			return;
		},
	};

	let guard = bot_data.apps.read().await;
	let mut choices: Vec<AutocompleteChoice> = Vec::with_capacity(result.len());

	// Truncate the name to 100 characters (Discord limit) and map to app id
	for (name, _) in result.into_iter() {
		if name.is_empty() {
			continue;
		}
		let name_show = if name.len() > 100 {
			name.chars().take(100).collect::<String>()
		} else {
			name.clone()
		};

		if let Some(app_id) = guard.get(name.as_str()) {
			choices.push(AutocompleteChoice::new(name_show, app_id.to_string()));
		}
	}

	let data = CreateAutocompleteResponse::new().set_choices(choices);
	let builder = CreateInteractionResponse::Autocomplete(data);

	if let Err(why) = autocomplete_interaction
		.create_response(&ctx.http, builder)
		.await
	{
		debug!("Error sending response: {:?}", why);
	}
}
