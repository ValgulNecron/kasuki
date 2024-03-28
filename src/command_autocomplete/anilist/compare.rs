use serenity::all::{
    AutocompleteChoice, CommandInteraction, Context, CreateAutocompleteResponse,
    CreateInteractionResponse,
};
use tracing::log::trace;

use crate::anilist_struct::autocomplete::user::UserPageWrapper;
use crate::command_run::get_option::get_option_map_string_autocomplete_subcommand;
use crate::constant::DEFAULT_STRING;

pub async fn autocomplete(ctx: Context, autocomplete_interaction: CommandInteraction) {
    let mut choice = Vec::new();
    trace!("{:?}", &autocomplete_interaction.data.options);
    let map = get_option_map_string_autocomplete_subcommand(&autocomplete_interaction);
    let user1 = map.get(&String::from("username")).unwrap_or(DEFAULT_STRING);
    choice.extend(get_choices(user1).await);
    let user2 = map
        .get(&String::from("username2"))
        .unwrap_or(DEFAULT_STRING);
    choice.extend(get_choices(user2).await);

    let data = CreateAutocompleteResponse::new().set_choices(choice);
    let builder = CreateInteractionResponse::Autocomplete(data);

    let _ = autocomplete_interaction
        .create_response(ctx.http, builder)
        .await;
}

async fn get_choices(search: &str) -> Vec<AutocompleteChoice> {
    trace!("{:?}", search);
    let data = UserPageWrapper::new_autocomplete_user(&search.to_string()).await;
    let mut choices = Vec::new();
    let users = data.data.page.users.unwrap().clone();

    for user in users {
        let data = user.unwrap();
        let user = data.name;
        choices.push(AutocompleteChoice::new(user, data.id.to_string()))
    }
    choices
}
