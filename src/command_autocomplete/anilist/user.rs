use serenity::all::{
    AutocompleteChoice, CommandInteraction, Context, CreateAutocompleteResponse,
    CreateInteractionResponse,
};

use crate::anilist_struct::autocomplete::user::UserPageWrapper;
use crate::command_run::get_option::get_option_map;
use crate::constant::DEFAULT_STRING;

pub async fn autocomplete(ctx: Context, autocomplete_interaction: CommandInteraction) {
    let map = get_option_map(&autocomplete_interaction);
    let user_search = map.get(&String::from("username")).unwrap_or(DEFAULT_STRING);
    let data = UserPageWrapper::new_autocomplete_user(user_search).await;
    let mut choices = Vec::new();
    let users = data.data.page.users.unwrap().clone();

    for user in users {
        let data = user.unwrap();
        let user = data.name;
        choices.push(AutocompleteChoice::new(user, data.id.to_string()))
    }

    let data = CreateAutocompleteResponse::new().set_choices(choices);
    let builder = CreateInteractionResponse::Autocomplete(data);

    let _ = autocomplete_interaction
        .create_response(ctx.http, builder)
        .await;
}
