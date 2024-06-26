use crate::config::Config;
use crate::helper::create_default_embed::get_default_embed;
use crate::helper::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use crate::helper::get_option::subcommand::get_option_map_string_subcommand;
use crate::helper::vndbapi::user::get_user;
use crate::structure::message::vn::user::load_localization_user;
use crate::structure::message::vn::user::UserLocalised;
use serenity::all::{
    CommandInteraction, Context, CreateInteractionResponse, CreateInteractionResponseMessage,
};
use std::sync::Arc;

pub async fn run(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    config: Arc<Config>,
) -> Result<(), AppError> {
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };
    let map = get_option_map_string_subcommand(command_interaction);
    let user = map.get(&String::from("username")).ok_or(AppError {
        message: String::from("No user found"),
        error_type: ErrorType::Command,
        error_response_type: ErrorResponseType::Message,
    })?;
    let path = format!("/user?q={}&fields=lengthvotes,lengthvotes_sum", user);
    let user = get_user(path).await?;
    let user_localised: UserLocalised = load_localization_user(guild_id).await?;
    let mut fields = vec![];
    fields.push((user_localised.id.clone(), user.id.clone(), true));
    fields.push((
        user_localised.playtime.clone(),
        user.lengthvotes.to_string(),
        true,
    ));
    fields.push((
        user_localised.playtimesum.clone(),
        user.lengthvotes_sum.to_string(),
        true,
    ));
    fields.push((user_localised.name.clone(), user.username.clone(), true));
    let builder_embed = get_default_embed(None)
        .title(user_localised.title)
        .fields(fields);
    let builder_message = CreateInteractionResponseMessage::new().embed(builder_embed);
    let builder = CreateInteractionResponse::Message(builder_message);
    command_interaction
        .create_response(&ctx.http, builder)
        .await
        .map_err(|e| {
            AppError::new(
                format!("Error while sending the command {}", e),
                ErrorType::Command,
                ErrorResponseType::Message,
            )
        })
}
