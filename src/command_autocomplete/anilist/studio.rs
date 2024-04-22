use serenity::all::{
    AutocompleteChoice, CommandInteraction, Context, CreateAutocompleteResponse,
    CreateInteractionResponse,
};

use crate::anilist_struct::autocomplete::studio::StudioPageWrapper;
use crate::common::get_option::subcommand::get_option_map_string_autocomplete_subcommand;
use crate::constant::DEFAULT_STRING;

pub async fn autocomplete(ctx: Context, autocomplete_interaction: CommandInteraction) {
    let map = get_option_map_string_autocomplete_subcommand(&autocomplete_interaction);
    let studio_search = map.get(&String::from("studio")).unwrap_or(DEFAULT_STRING);
    let data = StudioPageWrapper::new_autocomplete_staff(studio_search).await;
    let studios = data.data.page.studios.clone().unwrap();

    let mut choices = Vec::new();

    for studio in studios {
        let data = studio.unwrap();
        let user = data.name;
        choices.push(AutocompleteChoice::new(user, data.id.to_string()))
    }

    let data = CreateAutocompleteResponse::new().set_choices(choices);
    let builder = CreateInteractionResponse::Autocomplete(data);

    let _ = autocomplete_interaction
        .create_response(ctx.http, builder)
        .await;
}
