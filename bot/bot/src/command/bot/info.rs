use crate::command::embed_content::{CreateFooter, EmbedContent, EmbedsContents};
use crate::constant::{APP_VERSION, LIBRARY};
use crate::event_handler::BotData;
use anyhow::anyhow;
use kasuki_macros::slash_command;
use sea_orm::EntityTrait;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use shared::database::prelude::UserColor;
use shared::localization::{get_language_identifier, Loader, USABLE_LOCALES};
use tracing::{debug, info};

#[slash_command(
	name = "info", desc = "Get information on the bot.",
	command_type = SubCommand(parent = "bot"),
	contexts = [Guild, BotDm, PrivateChannel],
	install_contexts = [Guild, User],
)]
async fn info_command(self_: InfoCommand) -> Result<EmbedsContents<'_>> {
	info!("Processing info command");
	let ctx = self_.get_ctx();
	let bot_data = ctx.data::<BotData>().clone();
	let command_interaction = self_.get_command_interaction();
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

	// Get the language identifier for the guild
	let lang_id = get_language_identifier(guild_id, db_connection.clone()).await;

	// Retrieve various details about the bot and the server
	debug!("Retrieving bot and server details");
	let shard_count = ctx.cache.shard_count();
	debug!("Shard count: {}", shard_count);

	let shard = ctx.shard_id.to_string();
	debug!("Current shard: {}", shard);

	debug!("Retrieving user count");
	let user_count = UserColor::find().all(&*db_connection).await?.len();
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
	debug!(
		"Server count from cache: {}, from API: {}",
		server_count, app_guild_count
	);

	let guild_count = if server_count > app_guild_count {
		app_guild_count
	} else {
		server_count
	};
	debug!("Final guild count: {}", guild_count);

	let app_installation_count = bot.approximate_user_install_count.unwrap_or_default() as usize;
	debug!("App installation count: {}", app_installation_count);

	// Retrieve the bot's avatar
	debug!("Retrieving bot icon");
	let bot_icon = bot.icon.ok_or(anyhow!("No bot icon"))?;
	debug!("Bot icon retrieved");

	let avatar = if bot_icon.is_animated() {
		format!(
			"https://cdn.discordapp.com/app-icons/{}/{}.gif?size=1024",
			bot_id, bot_icon
		)
	} else {
		format!(
			"https://cdn.discordapp.com/app-icons/{}/{}.webp?size=1024",
			bot_id, bot_icon
		)
	};
	debug!("Avatar URL: {}", avatar);

	let lib = LIBRARY.to_string();
	debug!("Library: {}", lib);

	debug!("Creating embed content");
	let title = USABLE_LOCALES.lookup(&lang_id, "bot_info-title");
	let embed_content = EmbedContent::new(title.clone())
		.description(USABLE_LOCALES.lookup(&lang_id, "bot_info-desc"))
		.thumbnail(avatar)
		.fields(vec![
			(
				USABLE_LOCALES.lookup(&lang_id, "bot_info-bot_name"),
				bot_name,
				true,
			),
			(
				USABLE_LOCALES.lookup(&lang_id, "bot_info-bot_id"),
				bot_id,
				true,
			),
			(
				USABLE_LOCALES.lookup(&lang_id, "bot_info-version"),
				String::from(APP_VERSION),
				true,
			),
			(
				USABLE_LOCALES.lookup(&lang_id, "bot_info-shard_count"),
				shard_count.to_string(),
				true,
			),
			(
				USABLE_LOCALES.lookup(&lang_id, "bot_info-shard"),
				shard,
				true,
			),
			(
				USABLE_LOCALES.lookup(&lang_id, "bot_info-user_count"),
				user_count.to_string(),
				true,
			),
			(
				USABLE_LOCALES.lookup(&lang_id, "bot_info-server_count"),
				guild_count.to_string(),
				true,
			),
			(
				USABLE_LOCALES.lookup(&lang_id, "bot_info-creation_date"),
				creation_date,
				true,
			),
			(
				USABLE_LOCALES.lookup(&lang_id, "bot_info-library"),
				lib,
				true,
			),
			(
				USABLE_LOCALES.lookup(&lang_id, "bot_info-app_installation_count"),
				app_installation_count.to_string(),
				true,
			),
		])
		.footer(CreateFooter::new(
			USABLE_LOCALES.lookup(&lang_id, "bot_info-footer"),
		));
	debug!("Embed content created with title: {}", title);

	debug!("Creating final embed contents with buttons");
	let embed_contents = EmbedsContents::new(vec![embed_content]);

	info!("Info command processed successfully");
	Ok(embed_contents)
}
