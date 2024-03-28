use serenity::all::{
    CommandInteraction, Context, CreateInteractionResponse, CreateInteractionResponseMessage,
};

use crate::anilist_struct::run::user::{get_color, get_user_url, UserWrapper};
use crate::command_run::anilist::user::get_user_data;
use crate::command_run::get_option::get_option_map_string_subcommand;
use crate::common::default_embed::get_default_embed;
use crate::database::dispatcher::data_dispatch::set_registered_user;
use crate::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use crate::lang_struct::anilist::register::load_localization_register;

pub async fn run(ctx: &Context, command_interaction: &CommandInteraction) -> Result<(), AppError> {
    let map = get_option_map_string_subcommand(command_interaction);
    let value = map.get(&String::from("username")).ok_or(AppError::new(
        String::from("There is no option"),
        ErrorType::Option,
        ErrorResponseType::Followup,
    ))?;

    let data: UserWrapper = get_user_data(value).await?;

    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };

    let register_localised = load_localization_register(guild_id).await?;

    let user_data = data.data.user.clone();

    let user_id = &command_interaction.user.id.to_string();
    let username = &command_interaction.user.name;

    set_registered_user(user_id, &user_data.id.unwrap_or(0).to_string()).await?;

    let desc = register_localised
        .desc
        .replace("$user$", username.as_str())
        .replace("$id$", user_id)
        .replace(
            "$anilist$",
            user_data.name.clone().unwrap_or_default().as_str(),
        );

    let builder_embed = get_default_embed(Some(get_color(user_data.clone())))
        .title(user_data.name.unwrap_or_default())
        .url(get_user_url(user_data.id.unwrap_or(0)))
        .thumbnail(user_data.avatar.large.unwrap())
        .description(desc);

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
