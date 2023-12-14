use crate::anilist_struct::autocomplete::user::UserPageWrapper;
use serenity::all::{
    AutocompleteChoice, CommandInteraction, Context, CreateAutocompleteResponse,
    CreateInteractionResponse,
};

pub async fn autocomplete(ctx: Context, command: CommandInteraction) {
    let mut choice = Vec::new();
    match &command.data.options.get(0) {
        Some(_) => {
            let search = &command.data.options.get(0).unwrap().value.as_str().unwrap();
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
        None => {}
    }
    match &command.data.options.get(1) {
        Some(_) => {
            let search = &command.data.options.get(1).unwrap().value.as_str().unwrap();
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
        None => {}
    }

    let data = CreateAutocompleteResponse::new().set_choices(choice);
    let builder = CreateInteractionResponse::Autocomplete(data);

    let _ = command.create_response(ctx.http, builder).await;
}
