use std::error::Error;
use std::sync::Arc;

use serenity::all::{
    CommandInteraction, Context, CreateInteractionResponse, CreateInteractionResponseMessage, User,
};
use tokio::sync::{RwLock, RwLockReadGuard};

use crate::event_handler::{Handler, RootUsage};
use crate::helper::create_default_embed::get_default_embed;
use crate::helper::error_management::error_enum::ResponseError;
use crate::helper::get_option::subcommand::get_option_map_user_subcommand;
use crate::helper::get_user_data::get_user_data;
use crate::structure::message::user::command_usage::load_localization_command_usage;

pub async fn run(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    self_handler: &Handler,
) -> Result<(), Box<dyn Error>> {
    let db_type = self_handler.bot_data.config.bot.config.db_type.clone();
    let command_usage = self_handler
        .bot_data
        .number_of_command_use_per_command
        .clone();
    // Retrieve the user's name from the command interaction
    let map = get_option_map_user_subcommand(command_interaction);
    let user = map.get(&String::from("username"));
    // Check if the user exists
    match user {
        Some(user) => {
            let user = get_user_data(ctx.http.clone(), user).await?;
            command_usage_with_user(ctx, command_interaction, &user, command_usage, db_type).await
        }
        None => {
            // If the user does not exist, display the profile of the user who triggered the command
            command_usage_without_user(ctx, command_interaction, command_usage, db_type).await
        }
    }
}

async fn command_usage_without_user(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    command_usage: Arc<RwLock<RootUsage>>,
    db_type: String,
) -> Result<(), Box<dyn Error>> {
    // Retrieve the user who triggered the command
    let user = command_interaction.user.clone();
    // Display the user's profile
    command_usage_with_user(ctx, command_interaction, &user, command_usage, db_type).await
}

pub async fn command_usage_with_user(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    user: &User,
    command_usage: Arc<RwLock<RootUsage>>,
    db_type: String,
) -> Result<(), Box<dyn Error>> {
    send_embed(ctx, command_interaction, user, command_usage, db_type).await
}

pub async fn send_embed(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    user: &User,
    command_usage: Arc<RwLock<RootUsage>>,
    db_type: String,
) -> Result<(), Box<dyn Error>> {
    let id = user.id.to_string();
    let username = user.name.clone();
    let read_command_usage = command_usage.read().await;
    let usage = get_usage_for_id(&id, read_command_usage);
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };
    let command_usage_localised = load_localization_command_usage(guild_id, db_type).await?;
    let embed =
        get_default_embed(None).title(command_usage_localised.title.replace("$user$", &username));
    let mut embeds = Vec::new();
    if usage.is_empty() {
        let inner_embed = embed.description(
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
        .map_err(|e| ResponseError::Sending(format!("{:#?}", e)))?;
    Ok(())
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
