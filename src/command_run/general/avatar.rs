use serenity::all::{
    CommandInteraction, Context, CreateEmbed, CreateInteractionResponse,
    CreateInteractionResponseMessage, Timestamp, User,
};

use crate::common::get_option::subcommand::get_option_map_user_subcommand;
use crate::constant::COLOR;
use crate::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use crate::lang_struct::general::avatar::load_localization_avatar;

pub async fn run(ctx: &Context, command_interaction: &CommandInteraction) -> Result<(), AppError> {
    let map = get_option_map_user_subcommand(command_interaction);
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

pub async fn avatar_with_user(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    user: &User,
) -> Result<(), AppError> {
    let avatar_url = user.face();
    let guild_id = command_interaction.guild_id.unwrap_or_default();
    let user_id = user.id;
    let server_avatar = match guild_id.member(&ctx.http, user_id).await {
        Ok(member) => member.avatar_url(),
        Err(_) => None,
    };
    send_embed(
        avatar_url,
        ctx,
        command_interaction,
        user.name.clone(),
        server_avatar,
    )
    .await
}

pub async fn send_embed(
    avatar_url: String,
    ctx: &Context,
    command_interaction: &CommandInteraction,
    username: String,
    server_avatar: Option<String>,
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

    let builder_message = if server_avatar.is_none() {
        CreateInteractionResponseMessage::new().embed(builder_embed)
    } else {
        let second_builder_embed = CreateEmbed::new()
            .timestamp(Timestamp::now())
            .color(COLOR)
            .image(server_avatar.unwrap())
            .title(
                avatar_localised
                    .server_title
                    .replace("$user$", username.as_str()),
            );
        let embeds = vec![builder_embed, second_builder_embed];
        CreateInteractionResponseMessage::new().embeds(embeds)
    };

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
