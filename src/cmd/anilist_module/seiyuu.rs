use crate::function::error_management::common::custom_error;
use crate::structure::anilist::staff::struct_staff_image::StaffImageWrapper;
use serenity::builder::CreateApplicationCommand;
use serenity::client::Context;
use serenity::model::application::command::CommandOptionType;
use serenity::model::prelude::application_command::{
    ApplicationCommandInteraction, CommandDataOption, CommandDataOptionValue,
};

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
            match StaffImageWrapper::new_staff_by_id(value.parse().unwrap()).await {
                Ok(staff_wrapper) => staff_wrapper,
                Err(error) => {
                    custom_error(ctx, command, &error).await;
                    return;
                }
            }
        } else {
            match StaffImageWrapper::new_staff_by_search(value).await {
                Ok(staff_wrapper) => staff_wrapper,
                Err(error) => {
                    custom_error(ctx, command, &error).await;
                    return;
                }
            }
        };
    }
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("seiyuu")
        .description("Get an image of a seiyuu with 4 of the role.")
        .create_option(|option| {
            let option = option
                .name("seiyuu_name")
                .description("Name of the seiyuu.")
                .kind(CommandOptionType::String)
                .required(true)
                .set_autocomplete(true);
            option
        })
}
