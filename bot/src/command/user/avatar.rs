use crate::command::command_trait::{Command, SlashCommand, UserCommand};
use crate::config::Config;
use crate::helper::create_default_embed::get_default_embed;
use crate::helper::error_management::error_enum::ResponseError;
use crate::helper::get_option::subcommand::get_option_map_user_subcommand;
use crate::structure::message::user::avatar::load_localization_avatar;
use serenity::all::{
    CommandInteraction, Context, CreateInteractionResponse, CreateInteractionResponseMessage, User,
};
use std::error::Error;
use std::sync::Arc;

pub struct AvatarCommand {
    pub ctx: Context,
    pub command_interaction: CommandInteraction,
    pub config: Arc<Config>,
}

impl Command for AvatarCommand {
    fn get_ctx(&self) -> &Context {
        &self.ctx
    }
    fn get_command_interaction(&self) -> &CommandInteraction {
        &self.command_interaction
    }
}

impl SlashCommand for AvatarCommand {
    async fn run_slash(&self) -> Result<(), Box<dyn Error>> {
        let user = get_user_command(&self.ctx, &self.command_interaction).await?;
        send_embed(&self.ctx, &self.command_interaction, user, &self.config).await
    }
}

impl UserCommand for AvatarCommand {
    async fn run_user(&self) -> Result<(), Box<dyn Error>> {
        let user = get_user_command_user(&self.ctx, &self.command_interaction).await;
        send_embed(&self.ctx, &self.command_interaction, user, &self.config).await
    }
}

pub async fn get_user_command_user(
    ctx: &Context,
    command_interaction: &CommandInteraction,
) -> User {
    let users = &command_interaction.data.resolved.users;
    let mut user: Option<User> = None;
    let command_user = command_interaction.user.clone();
    for (user_id, u) in users {
        // If the user_id is not the same as the id of the user who invoked the command, assign the user to u and break the loop
        if user_id != &command_interaction.user.id {
            user = Some(u.clone());
            break;
        }
    }
    let user = user.unwrap_or(command_user);

    user.id.to_user(&ctx.http).await.unwrap_or(user)
}

pub async fn get_user_command(
    ctx: &Context,
    command_interaction: &CommandInteraction,
) -> Result<User, Box<dyn Error>> {
    let user = get_option_map_user_subcommand(command_interaction);
    let user = user.get(&String::from("username"));
    let user = match user {
        Some(user) => user
            .to_user(&ctx.http)
            .await
            .map_err(|e| ResponseError::Sending(format!("Failed to get user from ID: {:#?}", e)))?,
        None => command_interaction.user.clone(),
    };
    Ok(user)
}

pub async fn send_embed(
    ctx: &Context,
    interaction: &CommandInteraction,
    user: User,
    config: &Config,
) -> Result<(), Box<dyn Error>> {
    let guild_id = interaction
        .guild_id
        .map(|id| id.to_string())
        .unwrap_or_default();
    let avatar_url = user.face();
    let username = user.name;

    let avatar_localised = load_localization_avatar(guild_id, config.bot.config.clone()).await?;

    let embed = get_default_embed(None)
        .image(avatar_url)
        .title(avatar_localised.title.replace("$user$", username.as_str()));

    let server_avatar = match interaction.guild_id {
        Some(guild_id) => {
            let member = guild_id.member(&ctx.http, interaction.user.id).await;
            match member {
                Ok(member) => member.avatar_url(),
                Err(_) => None,
            }
        }
        None => None,
    };

    let message = match server_avatar {
        Some(server_avatar) => {
            let server_embed = get_default_embed(None).image(server_avatar).title(
                avatar_localised
                    .server_title
                    .replace("$user$", username.as_str()),
            );
            CreateInteractionResponseMessage::new().embeds(vec![embed, server_embed])
        }
        None => CreateInteractionResponseMessage::new().embed(embed),
    };

    interaction
        .create_response(&ctx.http, CreateInteractionResponse::Message(message))
        .await
        .map_err(|e| ResponseError::Sending(e.to_string()))?;
    Ok(())
}
