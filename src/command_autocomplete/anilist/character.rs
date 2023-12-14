use crate::anilist_struct::autocomplete::character::CharacterPageWrapper;
use serenity::all::{
    AutocompleteChoice, CommandInteraction, Context, CreateAutocompleteResponse,
    CreateInteractionResponse,
};

pub async fn autocomplete(ctx: Context, command: CommandInteraction) {
    let search = &command
        .data
        .options
        .first()
        .unwrap()
        .value
        .as_str()
        .unwrap();
    let data = CharacterPageWrapper::new_autocomplete_character(&search.to_string()).await;
    let mut choices = Vec::new();
    let character = data.data.page.characters.unwrap().clone();

    for user in character {
        let data = user.unwrap();
        let name = data.name.unwrap();
        let full = name.full.clone();
        let user_pref = name.user_preferred.clone();
        let name = user_pref.unwrap_or(full);
        choices.push(AutocompleteChoice::new(name, data.id.to_string()))
    }

    let data = CreateAutocompleteResponse::new().set_choices(choices);
    let builder = CreateInteractionResponse::Autocomplete(data);

    let _ = command.create_response(ctx.http, builder).await;
}
