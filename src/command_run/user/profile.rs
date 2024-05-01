use serenity::all::{
    CommandInteraction, Context, CreateEmbed, CreateInteractionResponse,
    CreateInteractionResponseMessage, Timestamp, User,
};

use crate::common::get_option::subcommand::get_option_map_user_subcommand;
use crate::constant::COLOR;
use crate::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use crate::lang_struct::user::profile::load_localization_profile;

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
            profile_with_user(ctx, command_interaction, &user).await
        }
        None => profile_without_user(ctx, command_interaction).await,
    }
}

async fn profile_without_user(
    ctx: &Context,
    command_interaction: &CommandInteraction,
) -> Result<(), AppError> {
    let user = command_interaction.user.clone();
    profile_with_user(ctx, command_interaction, &user).await
}

async fn profile_with_user(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    user: &User,
) -> Result<(), AppError> {
    let avatar_url = user.face();

    send_embed(avatar_url, ctx, command_interaction, user).await
}

pub async fn send_embed(
    avatar_url: String,
    ctx: &Context,
    command_interaction: &CommandInteraction,
    user: &User,
) -> Result<(), AppError> {
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };

    let profile_localised = load_localization_profile(guild_id).await?;

    let member = &command_interaction.member.clone().ok_or(AppError::new(
        String::from("There is no member in the option"),
        ErrorType::Option,
        ErrorResponseType::Message,
    ))?;

    let public_flag = match user.public_flags {
        Some(public_flag) => {
            let mut user_flags = Vec::new();
            for (flag, _) in public_flag.iter_names() {
                user_flags.push(flag)
            }
            user_flags.join(" / ")
        }
        None => "None".to_string(),
    };

    let builder_embed = CreateEmbed::new()
        .timestamp(Timestamp::now())
        .color(COLOR)
        .thumbnail(avatar_url)
        .title(
            profile_localised
                .title
                .replace("$user$", user.name.as_str()),
        )
        .description(
            profile_localised
                .desc
                .replace("$user$", user.name.as_str())
                .replace("$id$", user.id.to_string().as_str())
                .replace("$creation_date$", user.created_at().to_string().as_str())
                .replace(
                    "$joined_date$",
                    member
                        .joined_at
                        .ok_or(AppError::new(
                            String::from("There is no joined date for the user"),
                            ErrorType::Option,
                            ErrorResponseType::Message,
                        ))?
                        .to_string()
                        .as_str(),
                )
                .replace("$bot$", user.bot.to_string().as_str())
                .replace("$public_flag$", public_flag.as_str())
                .replace("$nitro$", format!("{:?}", user.premium_type).as_str())
                .replace("$system$", user.system.to_string().as_str()),
        );

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
