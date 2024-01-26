use serenity::all::{
    AutocompleteChoice, CommandInteraction, Context, CreateAutocompleteResponse,
    CreateInteractionResponse,
};

use crate::anilist_struct::autocomplete::staff::StaffPageWrapper;

pub async fn autocomplete(ctx: Context, autocomplete_interaction: CommandInteraction) {
    let mut search = String::new();
    for option in &autocomplete_interaction.data.options {
        if option.name.as_str() != "type" {
            search = option.value.as_str().unwrap().to_string()
        }
    }
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

    let _ = autocomplete_interaction
        .create_response(ctx.http, builder)
        .await;
}
