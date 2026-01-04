use crate::command::command::Command;
use crate::command::embed_content::{
    CommandType, CreateFooter, EmbedContent,
    EmbedsContents,
};
use crate::constant::{APP_VERSION, LIBRARY};
use crate::database::prelude::UserColor;
use crate::event_handler::BotData;
use crate::get_url;
use crate::impl_command;
use crate::structure::message::bot::info::load_localization_info;
use anyhow::anyhow;
use sea_orm::EntityTrait;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use tracing::{debug, info};

#[derive(Clone)]
pub struct InfoCommand {
    pub ctx: SerenityContext,
    pub command_interaction: CommandInteraction,
}

impl_command!(
	for InfoCommand,
	get_contents = |self_: InfoCommand| async move {
		info!("Processing info command");
		let ctx = self_.get_ctx();
		let bot_data = ctx.data::<BotData>().clone();
		let command_interaction = self_.get_command_interaction();
		let config = bot_data.config.clone();

		debug!("Retrieving bot data and configuration");

		// Retrieve the guild ID from the command interaction
		let guild_id = match command_interaction.guild_id {
			Some(id) => {
				debug!("Command executed in guild: {}", id);
				id.to_string()
			},
			None => {
				debug!("Command executed in DM");
				String::from("0")
			},
		};
		let db_connection = bot_data.db_connection.clone();

		// Load the localized information strings
		debug!("Loading info localization for guild: {}", guild_id);
		let info_localised = load_localization_info(guild_id, db_connection).await?;
		debug!("Info localization loaded successfully");

		// Retrieve various details about the bot and the server
		debug!("Retrieving bot and server details");
		let shard_count = ctx.cache.shard_count();
		debug!("Shard count: {}", shard_count);

		let shard = ctx.shard_id.to_string();
		debug!("Current shard: {}", shard);

		debug!("Connecting to database");
		let connection = sea_orm::Database::connect(get_url(config.db.clone())).await?;
		debug!("Database connection established");

		debug!("Retrieving user count");
		let user_count = UserColor::find().all(&connection).await?.len();
		debug!("User count: {}", user_count);

		debug!("Retrieving application info");
		let bot = ctx.http.get_current_application_info().await?;
		debug!("Application info retrieved");

		let bot_name = bot.name.to_string();
		let bot_id = bot.id.to_string();
		debug!("Bot name: {}, Bot ID: {}", bot_name, bot_id);

		let creation_date = format!("<t:{}:F>", bot.id.created_at().unix_timestamp());
		debug!("Bot creation date: {}", creation_date);

		let server_count = ctx.cache.guild_count();
		let app_guild_count = bot.approximate_guild_count.unwrap_or_default() as usize;
		debug!("Server count from cache: {}, from API: {}", server_count, app_guild_count);

		let guild_count = if server_count > app_guild_count {
			app_guild_count
		} else {
			server_count
		};
		debug!("Final guild count: {}", guild_count);

		let app_installation_count =
			bot.approximate_user_install_count.unwrap_or_default() as usize;
		debug!("App installation count: {}", app_installation_count);

		// Retrieve the bot's avatar
		debug!("Retrieving bot icon");
		let bot_icon = bot.icon.ok_or(anyhow!("No bot icon"))?;
		debug!("Bot icon retrieved");

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
		debug!("Avatar URL: {}", avatar);

		let lib = LIBRARY.to_string();
		debug!("Library: {}", lib);

		debug!("Creating embed content");
		let title = info_localised.title.clone();
		let embed_content = EmbedContent::new(info_localised.title)
			.description(info_localised.desc)
			.thumbnail(avatar)
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
			.footer(CreateFooter::new(info_localised.footer));
		debug!("Embed content created with title: {}", title);

		debug!("Creating final embed contents with buttons");
		let embed_contents = EmbedsContents::new(CommandType::First, vec![embed_content]);

		info!("Info command processed successfully");
		Ok(embed_contents)
	}
);
