use crate::command_run::get_option::get_option_map_user;
use serenity::all::{
    CommandInteraction, Context, CreateEmbed, CreateInteractionResponse,
    CreateInteractionResponseMessage, Timestamp, User,
};

use crate::constant::COLOR;
use crate::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use crate::lang_struct::general::avatar::load_localization_avatar;

pub async fn run(ctx: &Context, command_interaction: &CommandInteraction) -> Result<(), AppError> {
    let map = get_option_map_user(command_interaction);
    let user = map.get(&String::from("username"));

    match user {
        Some(user) => {
            let user = user.to_user(&ctx.http).await.map_err(|e| {
                AppError::new(
                    format!("Could not get the user. {}", e),
                    ErrorType::Option,
                    ErrorResponseType::Message,
                )
            })?;
            avatar_with_user(ctx, command_interaction, &user).await
        }
        None => avatar_without_user(ctx, command_interaction).await,
    }
}

async fn avatar_without_user(
    ctx: &Context,
    command_interaction: &CommandInteraction,
) -> Result<(), AppError> {
    let user = command_interaction.user.clone();
    avatar_with_user(ctx, command_interaction, &user).await
}

async fn avatar_with_user(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    user: &User,
) -> Result<(), AppError> {
    let avatar_url = user.face();
    send_embed(avatar_url, ctx, command_interaction, user.name.clone()).await
}

pub async fn send_embed(
    avatar_url: String,
    ctx: &Context,
    command_interaction: &CommandInteraction,
    username: String,
) -> Result<(), AppError> {
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };

    let avatar_localised = load_localization_avatar(guild_id).await?;

    let builder_embed = CreateEmbed::new()
        .timestamp(Timestamp::now())
        .color(COLOR)
        .image(avatar_url)
        .title(avatar_localised.title.replace("$user$", username.as_str()));

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
