//! The `AvatarCommand` struct handles avatar-related commands for users in Discord.
//!
//! It takes a Serenity context and a command interaction as input and processes
//! the command to provide appropriate responses based on the user's avatar.
use crate::command::command::{Command, CommandRun};
use crate::command::embed_content::{CommandType, EmbedContent, EmbedsContents};
use crate::event_handler::BotData;
use crate::helper::get_option::subcommand::get_option_map_user_subcommand;
use crate::impl_command;
use crate::structure::message::user::avatar::load_localization_avatar;
use anyhow::Result;
use serenity::all::{CommandInteraction, Context as SerenityContext, User};

/// A structure representing a command to fetch or handle avatar-related operations within a Discord bot.
///
/// # Fields
#[derive(Clone)]
pub struct AvatarCommand {
    pub ctx: SerenityContext,
    pub command_interaction: CommandInteraction,
}

impl_command!(
	for AvatarCommand,
	get_contents = |self_: AvatarCommand| async move {
		self_.defer().await?;
		let ctx = self_.get_ctx();
		let bot_data = ctx.data::<BotData>().clone();
		let command_interaction = self_.get_command_interaction();
		let db_connection = bot_data.db_connection.clone();
		let user = match command_interaction.data.kind.0 {
			1 => get_user_command(ctx, command_interaction).await?,
			2 => get_user_command_user(ctx, command_interaction).await,
			_ => {
				return Err(anyhow::anyhow!("Invalid command type"));
			},
		};

		let guild_id = command_interaction
			.guild_id
			.map(|id| id.to_string())
			.unwrap_or_default();
		let avatar_url = user.face();
		let username = user.name;
		let avatar_localised = load_localization_avatar(guild_id, db_connection).await?;
		let server_avatar = match command_interaction.guild_id {
			Some(guild_id) => {
				let member = guild_id
					.member(&ctx.http, command_interaction.user.id)
					.await;

				match member {
					Ok(member) => member.avatar_url(),
					Err(_) => None,
				}
			},
			None => None,
		};

		let title = avatar_localised.title.replace("$user$", username.as_str());
		let content1 = EmbedContent::new(title).images_url(avatar_url);

		let content2: Option<EmbedContent> = match server_avatar {
			Some(server_avatar) => {
				let title = avatar_localised
					.server_title
					.replace("$user$", username.as_str());
				let content2 = EmbedContent::new(title).images_url(server_avatar);

				Some(content2)
			},
			None => None,
		};

		let mut embed_content: Vec<EmbedContent> = vec![content1];
		if let Some(content2) = content2 {
			embed_content.push(content2);
		}

		let embed_contents = EmbedsContents::new(CommandType::Followup, embed_content);

		Ok(embed_contents)
	}
);

/// Retrieves a `User` object based on the provided [`CommandInteraction`] and [`SerenityContext`].
///
/// This function resolves the user associated with the command interaction.
///
/// - If any user is resolved from the `command_interaction.data.resolved.users` list and doesn't match
///   the invoking command user's ID, that user is returned.
/// - If no such user exists, it defaults to returning the user who invoked the command.
/// - Finally, the resolved user's full information is
pub async fn get_user_command_user(
    ctx: &SerenityContext, command_interaction: &CommandInteraction,
) -> User {
    let users = &command_interaction.data.resolved.users;

    let mut user: Option<User> = None;

    let command_user = command_interaction.user.clone();

    for user_inner in users {
        // If the user_id is not the same as the id of the user who invoked the command, assign the user to u and break the loop
        if user_inner.id.get() != command_interaction.user.id.get() {
            user = Some(user_inner.clone());

            break;
        }
    }

    let user = user.unwrap_or(command_user);

    user.id.to_user(&ctx.http).await.unwrap_or(user)
}

/// Asynchronously retrieves a `User` object based on the provided `CommandInteraction`.
///
/// This function extracts the user information from a subcommand option or
pub async fn get_user_command(
    ctx: &SerenityContext, command_interaction: &CommandInteraction,
) -> Result<User> {
    let user = get_option_map_user_subcommand(command_interaction);

    let user = user.get(&String::from("username"));

    let user = match user {
        Some(user) => user.to_user(&ctx.http).await?,
        None => command_interaction
            .user
            .id
            .to_user(&ctx.http)
            .await?
            .clone(),
    };

    Ok(user)
}
