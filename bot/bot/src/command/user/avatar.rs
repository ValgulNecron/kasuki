//! The `AvatarCommand` struct handles avatar-related commands for users in Discord.
//!
//! It takes a Serenity context and a command interaction as input and processes
//! the command to provide appropriate responses based on the user's avatar.
use crate::command::context::CommandContext;
use crate::command::embed_content::{EmbedContent, EmbedsContents};
use crate::helper::get_option::subcommand::get_option_map_user_subcommand;
use anyhow::Result;
use fluent_templates::fluent_bundle::FluentValue;
use kasuki_macros::slash_command;
use serenity::all::{CommandInteraction, Context as SerenityContext, User};
use shared::localization::{Loader, USABLE_LOCALES};
use std::borrow::Cow;
use std::collections::HashMap;

#[slash_command(
	name = "avatar", desc = "Get the avatar.",
	command_type = SubCommand(parent = "user"),
	contexts = [Guild, BotDm, PrivateChannel],
	install_contexts = [Guild, User],
	args = [(name = "username", desc = "Username of the user you want the avatar of.", arg_type = User, required = false, autocomplete = false)],
)]
async fn avatar_command(self_: AvatarCommand) -> Result<EmbedsContents<'_>> {
	let cx = CommandContext::new(
		self_.get_ctx().clone(),
		self_.get_command_interaction().clone(),
	);
	let user = match cx.command_interaction.data.kind.0 {
		1 => get_user_command(&cx.ctx, &cx.command_interaction).await?,
		2 => get_user_command_user(&cx.ctx, &cx.command_interaction).await,
		_ => {
			return Err(anyhow::anyhow!("Invalid command type"));
		},
	};

	let avatar_url = user.face();
	let username = user.name;
	let lang_id = cx.lang_id().await;
	let server_avatar = match cx.command_interaction.guild_id {
		Some(guild_id) => {
			let member = guild_id
				.member(&cx.ctx.http, cx.command_interaction.user.id)
				.await;

			match member {
				Ok(member) => member.avatar_url(),
				Err(_) => None,
			}
		},
		None => None,
	};

	let mut args: HashMap<Cow<'static, str>, FluentValue> = HashMap::new();
	args.insert(
		Cow::Borrowed("user"),
		FluentValue::from(username.to_string()),
	);
	let title = USABLE_LOCALES.lookup_with_args(&lang_id, "user_avatar-title", &args);
	let content1 = EmbedContent::new(title).images_url(avatar_url);

	let content2: Option<EmbedContent> = match server_avatar {
		Some(server_avatar) => {
			let mut args: HashMap<Cow<'static, str>, FluentValue> = HashMap::new();
			args.insert(
				Cow::Borrowed("user"),
				FluentValue::from(username.to_string()),
			);
			let title =
				USABLE_LOCALES.lookup_with_args(&lang_id, "user_avatar-server_title", &args);
			let content2 = EmbedContent::new(title).images_url(server_avatar);

			Some(content2)
		},
		None => None,
	};

	let mut embed_content: Vec<EmbedContent> = vec![content1];
	if let Some(content2) = content2 {
		embed_content.push(content2);
	}

	let embed_contents = EmbedsContents::new(embed_content);

	Ok(embed_contents)
}

#[slash_command(
	name = "avatar",
	command_type = User,
	struct_name = AvatarCommand,
	install_contexts = [Guild],
)]
async fn avatar_user_command(self_: AvatarCommand) -> Result<EmbedsContents<'_>> {
	unreachable!()
}

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

	let user = user.get("username");

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
