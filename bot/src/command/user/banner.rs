use std::sync::Arc;

use crate::command::command_trait::{Command, Embed, EmbedContent, SlashCommand, UserCommand};
use crate::command::user::avatar::{get_user_command, get_user_command_user};
use crate::config::{Config, DbConfig};
use crate::event_handler::BotData;
use crate::helper::create_default_embed::get_default_embed;
use crate::structure::message::user::banner::load_localization_banner;
use anyhow::Result;
use serenity::all::{
	CommandInteraction, Context as SerenityContext, CreateInteractionResponse,
	CreateInteractionResponseMessage, User,
};
pub struct BannerCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl Command for BannerCommand {
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}
}

impl SlashCommand for BannerCommand {
	async fn run_slash(&self) -> Result<()> {
		let user = get_user_command(&self.ctx, &self.command_interaction).await?;
		let ctx = self.get_ctx();
		let bot_data = ctx.data::<BotData>().clone();
		let config = bot_data.config.clone();
		let command_interaction = self.get_command_interaction();

		let db_config = config.db.clone();

		let banner = match user.banner_url() {
			Some(url) => url,
			None => {
				no_banner(command_interaction, &user.name, db_config).await?;

				return Ok(());
			},
		};

		let username = user.name.as_str();

		let guild_id = match command_interaction.guild_id {
			Some(id) => id.to_string(),
			None => String::from("0"),
		};

		let banner_localised = load_localization_banner(guild_id, db_config).await?;

		let content = EmbedContent::new(banner_localised.title.replace("$user$", username))
			.images_url(Some(banner));
		self.send_embed(vec![content]).await
	}
}

impl UserCommand for BannerCommand {
	async fn run_user(&self) -> Result<()> {
		let user = get_user_command_user(&self.ctx, &self.command_interaction).await;
		let ctx = self.get_ctx();
		let bot_data = ctx.data::<BotData>().clone();
		let config = bot_data.config.clone();
		let command_interaction = self.get_command_interaction();

		let db_config = config.db.clone();

		let banner = match user.banner_url() {
			Some(url) => url,
			None => {
				no_banner(command_interaction, &user.name, db_config).await?;

				return Ok(());
			},
		};

		let username = user.name.as_str();

		let guild_id = match command_interaction.guild_id {
			Some(id) => id.to_string(),
			None => String::from("0"),
		};

		let banner_localised = load_localization_banner(guild_id, db_config).await?;

		let content = EmbedContent::new(banner_localised.title.replace("$user$", username))
			.images_url(Some(banner));
		self.send_embed(vec![content]).await
	}
}

pub async fn no_banner(
	command_interaction: &CommandInteraction, username: &str, db_config: DbConfig,
) -> Result<Vec<EmbedContent<'static, 'static>>> {
	let guild_id = match command_interaction.guild_id {
		Some(id) => id.to_string(),
		None => String::from("0"),
	};

	let banner_localised = load_localization_banner(guild_id, db_config).await?;

	let content = EmbedContent::new(banner_localised.no_banner_title)
		.description(banner_localised.no_banner.replace("$user$", username));

	Ok(vec![content])
}
