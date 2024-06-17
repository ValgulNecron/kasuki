use serenity::all::{
    AutocompleteChoice, CommandInteraction, Context, CreateAutocompleteResponse,
    CreateInteractionResponse,
};
use tracing::trace;

use crate::helper::get_option::subcommand::get_option_map_string_autocomplete_subcommand;
use crate::helper::vndbapi::character::{Character, CharacterRoot, get_character};
use crate::helper::vndbapi::game::VN;

pub async fn autocomplete(ctx: Context, autocomplete_interaction: CommandInteraction) {
    let map = get_option_map_string_autocomplete_subcommand(&autocomplete_interaction);
    let game = map.get(&String::from("name")).unwrap();
    let vn = get_character(game.clone()).await.unwrap();
    let vn_result = vn.results;
    // take the 25 first results
    let characters: Vec<Character> = vn_result.iter().take(25).cloned().collect();
    let mut choices = Vec::new();
    trace!("Game: {}", game);
    trace!("Map: {:?}", map);
    for character in characters {
        choices.push(AutocompleteChoice::new(character.name.clone(), character.id.clone()))
    }

    let data = CreateAutocompleteResponse::new().set_choices(choices);
    let builder = CreateInteractionResponse::Autocomplete(data);

    let _ = autocomplete_interaction
        .create_response(ctx.http, builder)
        .await;
}
