use crate::constant::COLOR;
use crate::function::error_management::common::custom_error;
use crate::structure::anilist::staff::struct_autocomplete_staff::StaffPageWrapper;
use crate::structure::anilist::staff::struct_staff::StaffWrapper;
use crate::structure::embed::anilist::struct_lang_staff::StaffLocalisedText;
use crate::structure::register::anilist::struct_staff_register::RegisterLocalisedStaff;
use serde_json::json;
use serenity::builder::CreateApplicationCommand;
use serenity::client::Context;
use serenity::model::application::interaction::application_command::CommandDataOptionValue;
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::prelude::autocomplete::AutocompleteInteraction;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::interaction::application_command::{
    ApplicationCommandInteraction, CommandDataOption,
};
use serenity::model::Timestamp;

pub async fn run(
    options: &[CommandDataOption],
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) {
    let option = options
        .get(0)
        .expect("Expected name option")
        .resolved
        .as_ref()
        .expect("Expected name object");
    if let CommandDataOptionValue::String(value) = option {
        let data = if value.parse::<i32>().is_ok() {
            match StaffWrapper::new_staff_by_id(value.parse().unwrap()).await {
                Ok(user_wrapper) => user_wrapper,
                Err(error) => {
                    custom_error(ctx, command, &error).await;
                    return;
                }
            }
        } else {
            match StaffWrapper::new_staff_by_search(value).await {
                Ok(user_wrapper) => user_wrapper,
                Err(error) => {
                    custom_error(ctx, command, &error).await;
                    return;
                }
            }
        };

        let staff_url = data.get_url();

        let staff_name = data.get_name();

        let image = data.get_image();

        let result_role: String = data.format_role();

        let result_va: String = data.format_va();

        let localised_text = match StaffLocalisedText::get_staff_localised(ctx, command).await {
            Ok(data) => data,
            Err(_) => return,
        };
        let desc = data.get_desc(&localised_text);

        if let Err(why) = command
            .create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| {
                        message.embed(|m| {
                            m.title(staff_name)
                                .timestamp(Timestamp::now())
                                .color(COLOR)
                                .fields(vec![
                                    (&localised_text.desc_title, desc, false),
                                    (&localised_text.media, result_role, true),
                                    (&localised_text.va, result_va, true),
                                ])
                                .url(staff_url)
                                .image(image)
                        })
                    })
            })
            .await
        {
            println!("{}: {}", localised_text.error_slash_command, why);
        }
    }
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    let staffs = RegisterLocalisedStaff::get_staff_register_localised().unwrap();
    let command = command
        .name("staff")
        .description("Get info of a staff")
        .create_option(|option| {
            let option = option
                .name("staff_name")
                .description("Name of the staff you want info about.")
                .kind(CommandOptionType::String)
                .required(true)
                .set_autocomplete(true);
            for staff in staffs.values() {
                option
                    .name_localized(&staff.code, &staff.option1)
                    .description_localized(&staff.code, &staff.option1_desc);
            }
            option
        });
    for staff in staffs.values() {
        command
            .name_localized(&staff.code, &staff.name)
            .description_localized(&staff.code, &staff.desc);
    }
    command
}

pub async fn autocomplete(ctx: Context, command: AutocompleteInteraction) {
    let search = &command.data.options.first().unwrap().value;
    if let Some(search) = search {
        let data = StaffPageWrapper::new_autocomplete_staff(search, 8).await;

        let choices = data.get_choice();
        // doesn't matter if it errors
        let choices_json = json!(choices);
        _ = command
            .create_autocomplete_response(ctx.http.clone(), |response| {
                response.set_choices(choices_json)
            })
            .await;
    }
}
