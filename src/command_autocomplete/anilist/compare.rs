use serenity::all::{
    AutocompleteChoice, CommandInteraction, Context, CreateAutocompleteResponse,
    CreateInteractionResponse,
};
use tracing::log::trace;

use crate::anilist_struct::autocomplete::user::UserPageWrapper;

pub async fn autocomplete(ctx: Context, command: CommandInteraction) {
    let mut choice = Vec::new();
    trace!("{:?}", &command.data.options);
    for option in &command.data.options {
        let search = option.value.as_str().unwrap();
        trace!("{:?}", search);
        let data = UserPageWrapper::new_autocomplete_user(&search.to_string()).await;
        let mut choices = Vec::new();
        let users = data.data.page.users.unwrap().clone();

        for user in users {
            let data = user.unwrap();
            let user = data.name;
            choices.push(AutocompleteChoice::new(user, data.id.to_string()))
        }
        choice.extend(choices);
    }

    let data = CreateAutocompleteResponse::new().set_choices(choice);
    let builder = CreateInteractionResponse::Autocomplete(data);

    let _ = command.create_response(ctx.http, builder).await;
}
