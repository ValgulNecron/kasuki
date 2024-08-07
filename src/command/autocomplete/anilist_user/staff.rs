use std::sync::Arc;

use cynic::{GraphQlResponse, QueryBuilder};
use moka::future::Cache;
use serenity::all::{
    AutocompleteChoice, CommandInteraction, Context, CreateAutocompleteResponse,
    CreateInteractionResponse,
};
use tokio::sync::RwLock;

use crate::constant::DEFAULT_STRING;
use crate::helper::get_option::subcommand::get_option_map_string_autocomplete_subcommand;
use crate::helper::make_graphql_cached::make_request_anilist;
use crate::structure::autocomplete::anilist::staff::{
    StaffAutocomplete, StaffAutocompleteVariables,
};

/// `autocomplete` is an asynchronous function that handles the autocomplete feature for staff search.
/// It takes a `Context` and a `CommandInteraction` as parameters.
/// `ctx` is the context in which this function is called.
/// `autocomplete_interaction` is the command interaction that triggered this function.
///
/// This function first gets the map of options from the command interaction.
/// It then gets the staff name from the map of options.
/// If the staff name is not found in the map, it defaults to a predefined string.
/// It then creates a new `StaffPageWrapper` for the autocomplete staff search with the staff name.
/// It gets the staff from the `StaffPageWrapper`.
/// For each staff, it gets the preferred name or the full name if the preferred name is not available.
/// It creates a new `AutocompleteChoice` with the name and the ID and pushes it to the vector.
/// It creates a new `CreateAutocompleteResponse` with the choices.
/// It creates a new `CreateInteractionResponse` with the `CreateAutocompleteResponse`.
/// It sends the response to the Discord channel.
///
/// # Arguments
///
/// * `ctx` - The context in which this function is called.
/// * `autocomplete_interaction` - The command interaction that triggered this function.
///
/// # Async
///
/// This function is asynchronous. It awaits the creation of the `StaffPageWrapper` and the sending of the response.
pub async fn autocomplete(
    ctx: Context,
    autocomplete_interaction: CommandInteraction,
    anilist_cache: Arc<RwLock<Cache<String, String>>>,
) {
    let map = get_option_map_string_autocomplete_subcommand(&autocomplete_interaction);
    let staff_search = map
        .get(&String::from("staff_name"))
        .unwrap_or(DEFAULT_STRING);
    let var = StaffAutocompleteVariables {
        search: Some(staff_search),
    };
    let operation = StaffAutocomplete::build(var);
    let data: GraphQlResponse<StaffAutocomplete> =
        match make_request_anilist(operation, false, anilist_cache).await {
            Ok(data) => data,
            Err(e) => {
                tracing::error!(?e);
                return;
            }
        };
    let mut choices = Vec::new();
    let staffs = match data.data {
        Some(data) => match data.page {
            Some(page) => match page.staff {
                Some(staff) => staff,
                None => {
                    return;
                }
            },
            None => {
                return;
            }
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
        .create_response(ctx.http, builder)
        .await;
}
