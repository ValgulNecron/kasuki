use serenity::all::CreateInteractionResponse::Defer;
use serenity::all::{CommandInteraction, Context, CreateInteractionResponseMessage};

use crate::command_run::anime::random_image::send_embed;
use crate::common::get_option::subcommand::get_option_map_string_subcommand;
use crate::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use crate::lang_struct::anilist::random_image_nsfw::load_localization_random_image_nsfw;

pub async fn run(ctx: &Context, command_interaction: &CommandInteraction) -> Result<(), AppError> {
    let map = get_option_map_string_subcommand(command_interaction);
    let image_type = map.get(&String::from("image_type")).ok_or(AppError::new(
        String::from("There is no option"),
        ErrorType::Option,
        ErrorResponseType::Followup,
    ))?;
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };
    let random_image_nsfw_localised = load_localization_random_image_nsfw(guild_id).await?;

    let builder_message = Defer(CreateInteractionResponseMessage::new());

    command_interaction
        .create_response(&ctx.http, builder_message)
        .await
        .map_err(|e| {
            AppError::new(
                format!("Error while sending the command {}", e),
                ErrorType::Command,
                ErrorResponseType::Message,
            )
        })?;
    send_embed(
        ctx,
        command_interaction,
        image_type,
        random_image_nsfw_localised.title,
        "nsfw",
    )
    .await
}
