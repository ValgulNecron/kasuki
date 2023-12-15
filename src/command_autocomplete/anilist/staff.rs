use crate::anilist_struct::autocomplete::staff::StaffPageWrapper;
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
    let data = StaffPageWrapper::new_autocomplete_staff(&search.to_string()).await;
    let mut choices = Vec::new();
    let staffs = data.data.page.staff.clone().unwrap();

    for staff in staffs {
        let data = staff.unwrap();
        let user = data
            .name
            .user_preferred
            .clone()
            .unwrap_or(data.name.full.clone());
        choices.push(AutocompleteChoice::new(user, data.id.to_string()))
    }

    let data = CreateAutocompleteResponse::new().set_choices(choices);
    let builder = CreateInteractionResponse::Autocomplete(data);

    let _ = command.create_response(ctx.http, builder).await;
}
