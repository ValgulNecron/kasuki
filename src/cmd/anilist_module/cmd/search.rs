use serenity::builder::CreateApplicationCommand;
use serenity::client::Context;
use serenity::model::application::interaction::application_command::CommandDataOptionValue;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::interaction::application_command::{
    ApplicationCommandInteraction, CommandDataOption,
};
use serenity::utils::Colour;

use crate::cmd::anilist_module::cmd::{anime, character, ln, manga, staff, studio, user};
use crate::cmd::error_modules::error_not_implemented::error_not_implemented;

pub async fn run(
    options: &[CommandDataOption],
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) {
    // Get the content of the first option.
    let option = options
        .get(1)
        .expect("Expected type option")
        .resolved
        .as_ref()
        .expect("Expected type object");
    // Check if the option variable contain the correct value.
    if let CommandDataOptionValue::String(search_type) = option {
        let search_types = search_type.as_ref();
        match search_types {
            "anime" => anime::run(&command.data.options, &ctx, &command).await,
            "character" => character::run(&command.data.options, &ctx, &command).await,
            "ln" => ln::run(&command.data.options, &ctx, &command).await,
            "manga" => manga::run(&command.data.options, &ctx, &command).await,
            "staff" => staff::run(&command.data.options, &ctx, &command).await,
            "user" => user::run(&command.data.options, &ctx, &command).await,
            "studio" => studio::run(&command.data.options, &ctx, &command).await,
            _ => {
                let color = Colour::FABLED_PINK;
                error_not_implemented(color, ctx, command).await
            }
        };
    }
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("search")
        .description("Info of an anime")
        .create_option(|option| {
            option
                .name("name")
                .description("The name of the anime/user/ln/etc...")
                .kind(CommandOptionType::String)
                .required(true)
        })
        .create_option(|option| {
            option
                .name("search_type")
                .description("The type of the search you want.")
                .kind(CommandOptionType::String)
                .add_string_choice("anime", "anime")
                .add_string_choice("character", "character")
                .add_string_choice("ln", "ln")
                .add_string_choice("manga", "manga")
                .add_string_choice("staff", "staff")
                .add_string_choice("user", "user")
                .add_string_choice("studio", "studio")
                .required(true)
        })
}
