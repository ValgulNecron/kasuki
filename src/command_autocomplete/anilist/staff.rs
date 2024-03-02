use serenity::all::{
    AutocompleteChoice, CommandInteraction, Context, CreateAutocompleteResponse,
    CreateInteractionResponse,
};

use crate::anilist_struct::autocomplete::staff::StaffPageWrapper;
use crate::command_run::get_option::{
    get_option_map_string_autocomplete_subcommand, get_option_map_string_subcommand,
};
use crate::constant::DEFAULT_STRING;

pub async fn autocomplete(ctx: Context, autocomplete_interaction: CommandInteraction) {
    let map = get_option_map_string_autocomplete_subcommand(&autocomplete_interaction);
    let staff_search = map
        .get(&String::from("staff_name"))
        .unwrap_or(DEFAULT_STRING);
    let data = StaffPageWrapper::new_autocomplete_staff(staff_search).await;
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
