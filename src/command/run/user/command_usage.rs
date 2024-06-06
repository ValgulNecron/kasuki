use serenity::all::{
    CommandInteraction, Context, CreateInteractionResponse, CreateInteractionResponseMessage, User,
};
use tokio::sync::RwLockReadGuard;

use crate::event_handler::RootUsage;
use crate::helper::create_normalise_embed::get_default_embed;
use crate::helper::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use crate::helper::get_option::subcommand::get_option_map_user_subcommand;
use crate::helper::get_user_data::get_user_data;
use crate::struct_shard_manager::RootUsageContainer;
use crate::structure::message::user::command_usage::load_localization_command_usage;

pub async fn run(ctx: &Context, command_interaction: &CommandInteraction) -> Result<(), AppError> {
    // Retrieve the user's name from the command interaction
    let map = get_option_map_user_subcommand(command_interaction);
    let user = map.get(&String::from("username"));

    // Check if the user exists
    match user {
        Some(user) => {
            let user = get_user_data(ctx.http.clone(), user).await?;
            command_usage_with_user(ctx, command_interaction, &user).await
        }
        None => {
            // If the user does not exist, display the profile of the user who triggered the command
            command_usage_without_user(ctx, command_interaction).await
        }
    }
}

async fn command_usage_without_user(
    ctx: &Context,
    command_interaction: &CommandInteraction,
) -> Result<(), AppError> {
    // Retrieve the user who triggered the command
    let user = command_interaction.user.clone();
    // Display the user's profile
    command_usage_with_user(ctx, command_interaction, &user).await
}

pub async fn command_usage_with_user(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    user: &User,
) -> Result<(), AppError> {
    send_embed(ctx, command_interaction, user).await
}

pub async fn send_embed(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    user: &User,
) -> Result<(), AppError> {
    let id = user.id.to_string();
    let username = user.name.clone();
    let commande_usage = ctx
        .data
        .read()
        .await
        .get::<RootUsageContainer>()
        .unwrap()
        .clone();
    let read_commande_usage = commande_usage.read().await;
    let usage = get_usage_for_id(&id, read_commande_usage);
    let embed = get_default_embed(None);
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };
    let mut embeds = Vec::new();
    let command_usage_localised = load_localization_command_usage(guild_id).await?;
    if usage.is_empty() {
        let inner_embed = embed
            .title(command_usage_localised.title.replace("$user$", &username))
            .description(
                command_usage_localised
                    .no_usage
                    .replace("$user$", &username),
            );
        embeds.push(inner_embed);
    } else {
        let mut description = String::new();
        let mut inner_embed = embed.clone();
        for (command, usage) in &usage {
            description.push_str(
                command_usage_localised
                    .command_usage
                    .replace("$command$", command)
                    .replace("$usage$", &usage.to_string())
                    .as_str(),
            );
            description.push('\n');
            if description.len() > 4096 {
                embeds.push(inner_embed.clone().description(&description));
                description.clear();
                inner_embed = embed.clone();
            }
        }
        if !description.is_empty() {
            embeds.push(inner_embed.clone().description(&description));
        }
    }
    let builder_message = CreateInteractionResponseMessage::new().embeds(embeds);

    let builder = CreateInteractionResponse::Message(builder_message);

    command_interaction
        .create_response(&ctx.http, builder)
        .await
        .map_err(|e| {
            AppError::new(
                format!("Error while sending the command {}", e),
                ErrorType::Command,
                ErrorResponseType::Followup,
            )
        })
}

fn get_usage_for_id(
    target_id: &str,
    root_usage: RwLockReadGuard<RootUsage>,
) -> Vec<(String, u128)> {
    let mut usage = Vec::new();
    for (command, user_info) in root_usage.command_list.iter() {
        for (id, user_usage) in user_info.user_info.iter() {
            if id == target_id {
                usage.push((command.clone(), user_usage.usage))
            }
        }
    }
    usage
}
