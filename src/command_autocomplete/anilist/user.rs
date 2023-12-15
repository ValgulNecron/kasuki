use crate::anilist_struct::autocomplete::user::UserPageWrapper;
use serenity::all::{
    AutocompleteChoice, CommandInteraction, Context, CreateAutocompleteResponse,
    CreateInteractionResponse,
};

pub async fn autocomplete(ctx: Context, command: CommandInteraction) {
    let mut search = String::new();
    for option in &command.data.options {
        if option.name.as_str() != "type" {
            search = option.value.as_str().unwrap().to_string()
        }
    }
    let data = UserPageWrapper::new_autocomplete_user(&search.to_string()).await;
    let mut choices = Vec::new();
    let users = data.data.page.users.unwrap().clone();

    for user in users {
        let data = user.unwrap();
        let user = data.name;
        choices.push(AutocompleteChoice::new(user, data.id.to_string()))
    }

    let data = CreateAutocompleteResponse::new().set_choices(choices);
    let builder = CreateInteractionResponse::Autocomplete(data);

    let _ = command.create_response(ctx.http, builder).await;
}
