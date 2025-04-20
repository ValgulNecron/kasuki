use crate::command::command_trait::{Command, Embed, EmbedContent, EmbedType, SlashCommand};
use crate::constant::{APP_VERSION, LIBRARY};
use crate::database::prelude::UserColor;
use crate::event_handler::BotData;
use crate::get_url;
use crate::structure::message::bot::info::load_localization_info;
use anyhow::{Result, anyhow};
use sea_orm::EntityTrait;
use serenity::all::{
	ButtonStyle, CommandInteraction, Context as SerenityContext, CreateActionRow, CreateButton,
};
use std::borrow::Cow;

pub struct InfoCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl Command for InfoCommand {
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}
}

impl SlashCommand for InfoCommand {
	async fn run_slash(&self) -> Result<()> {
		let ctx = self.get_ctx();
		let bot_data = ctx.data::<BotData>().clone();
		let command_interaction = self.get_command_interaction();
		let config = bot_data.config.clone();

		// Retrieve the guild ID from the command interaction
		let guild_id = match command_interaction.guild_id {
			Some(id) => id.to_string(),
			None => String::from("0"),
		};

		// Load the localized information strings
		let info_localised = load_localization_info(guild_id, config.db.clone()).await?;

		// Retrieve various details about the bot and the server
		let shard_count = ctx.cache.shard_count();

		let shard = ctx.shard_id.to_string();

		let connection = sea_orm::Database::connect(get_url(config.db.clone())).await?;

		let user_count = UserColor::find().all(&connection).await?.len();

		let bot = ctx.http.get_current_application_info().await?;

		let bot_name = bot.name.to_string();

		let bot_id = bot.id.to_string();

		let creation_date = format!("<t:{}:F>", bot.id.created_at().unix_timestamp());

		let server_count = ctx.cache.guild_count();

		let app_guild_count = bot.approximate_guild_count.unwrap_or_default() as usize;

		let guild_count = if server_count > app_guild_count {
			server_count
		} else {
			app_guild_count
		};

		let app_installation_count =
			bot.approximate_user_install_count.unwrap_or_default() as usize;

		// Retrieve the bot's avatar
		let bot_icon = bot.icon.ok_or(anyhow!("No bot icon"))?;

		let avatar = if bot_icon.is_animated() {
			format!(
				"https://cdn.discordapp.com/icons/{}/{}.gif?size=1024",
				bot_id, bot_icon
			)
		} else {
			format!(
				"https://cdn.discordapp.com/icons/{}/{}.webp?size=1024",
				bot_id, bot_icon
			)
		};

		let lib = LIBRARY.to_string();

		let mut buttons = Cow::from(vec![]);

		let mut components = vec![];

		// Add buttons for various actions

		buttons.to_mut().push(
			CreateButton::new_link("https://github.com/ValgulNecron/kasuki")
				.style(ButtonStyle::Primary)
				.label(info_localised.button_see_on_github),
		);

		buttons.to_mut().push(
			CreateButton::new_link("https://kasuki.valgul.moe/")
				.style(ButtonStyle::Primary)
				.label(info_localised.button_official_website),
		);

		buttons.to_mut().push(
			CreateButton::new_link("https://discord.gg/h4hYxMURQx")
				.style(ButtonStyle::Primary)
				.label(info_localised.button_official_discord),
		);

		components.push(CreateActionRow::Buttons(buttons.clone()));

		buttons.to_mut().clear();

		buttons.to_mut().push(
            CreateButton::new_link("https://discord.com/api/oauth2/authorize?client_id=923286536445894697&permissions=395677134144&scope=bot")
                .style(ButtonStyle::Primary)
                .label(info_localised.button_add_the_bot)
        );

		buttons.to_mut().push(
            CreateButton::new_link("https://discord.com/api/oauth2/authorize?client_id=1122304053620260924&permissions=395677134144&scope=bot")
                .style(ButtonStyle::Primary)
                .label(info_localised.button_add_the_beta_bot)
        );

		components.push(CreateActionRow::Buttons(buttons));

		let content = EmbedContent::new(info_localised.title)
			.description(info_localised.desc)
			.thumbnail(Some(avatar))
			.command_type(EmbedType::First)
			.fields(vec![
				(info_localised.bot_name, bot_name, true),
				(info_localised.bot_id, bot_id, true),
				(info_localised.version, String::from(APP_VERSION), true),
				(info_localised.shard_count, shard_count.to_string(), true),
				(info_localised.shard, shard, true),
				(info_localised.user_count, user_count.to_string(), true),
				(info_localised.server_count, guild_count.to_string(), true),
				(info_localised.creation_date, creation_date, true),
				(info_localised.library, lib, true),
				(
					info_localised.app_installation_count,
					app_installation_count.to_string(),
					true,
				),
			])
			.action_row(components)
			.footer(Some(info_localised.footer));

		self.send_embed(vec![content]).await
	}
}
