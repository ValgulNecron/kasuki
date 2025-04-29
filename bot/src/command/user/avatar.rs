use crate::command::command_trait::{Command, Embed, EmbedContent, SlashCommand, UserCommand};
use crate::config::Config;
use crate::event_handler::BotData;
use crate::helper::get_option::subcommand::get_option_map_user_subcommand;
use crate::structure::message::user::avatar::load_localization_avatar;
use anyhow::Result;
use serenity::all::{CommandInteraction, Context as SerenityContext, User};

pub struct AvatarCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl Command for AvatarCommand {
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}
}

impl SlashCommand for AvatarCommand {
	async fn run_slash(&self) -> Result<()> {
		let user = get_user_command(&self.ctx, &self.command_interaction).await?;
		let ctx = self.get_ctx();
		let bot_data = ctx.data::<BotData>().clone();
		let contents =
			get_user_content(&self.ctx, &self.command_interaction, user, &bot_data.config).await?;

		self.send_embed(contents).await
	}
}

impl UserCommand for AvatarCommand {
	async fn run_user(&self) -> Result<()> {
		let user = get_user_command_user(&self.ctx, &self.command_interaction).await;
		let ctx = self.get_ctx();
		let bot_data = ctx.data::<BotData>().clone();
		let contents =
			get_user_content(&self.ctx, &self.command_interaction, user, &bot_data.config).await?;

		self.send_embed(contents).await
	}
}

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

pub async fn get_user_content(
	ctx: &SerenityContext, interaction: &CommandInteraction, user: User, config: &Config,
) -> Result<Vec<EmbedContent<'static, 'static>>> {
	let guild_id = interaction
		.guild_id
		.map(|id| id.to_string())
		.unwrap_or_default();

	let avatar_url = user.face();

	let username = user.name;

	let avatar_localised = load_localization_avatar(guild_id, config.db.clone()).await?;

	let server_avatar = match interaction.guild_id {
		Some(guild_id) => {
			let member = guild_id.member(&ctx.http, interaction.user.id).await;

			match member {
				Ok(member) => member.avatar_url(),
				Err(_) => None,
			}
		},
		None => None,
	};

	let content1 = EmbedContent::new(avatar_localised.title.replace("$user$", username.as_str()))
		.images_url(Some(avatar_url));

	let contents = match server_avatar {
		Some(server_avatar) => {
			let content2 = EmbedContent::new(
				avatar_localised
					.server_title
					.replace("$user$", username.as_str()),
			)
			.images_url(Some(server_avatar));

			vec![content1, content2]
		},
		None => vec![content1],
	};

	Ok(contents)
}
