use crate::command::context::CommandContext;
use crate::command::embed_content::{CreateFooter, EmbedContent, EmbedsContents};
use crate::constant::{APP_VERSION, LIBRARY};
use anyhow::anyhow;
use kasuki_macros::slash_command;
use sea_orm::{EntityTrait, PaginatorTrait};
use serenity::all::{CommandInteraction, Context as SerenityContext};
use shared::database::prelude::UserColor;
use shared::localization::{Loader, USABLE_LOCALES};

#[slash_command(
	name = "info", desc = "Get information on the bot.",
	command_type = SubCommand(parent = "bot"),
	contexts = [Guild, BotDm, PrivateChannel],
	install_contexts = [Guild, User],
)]
async fn info_command(self_: InfoCommand) -> Result<EmbedsContents<'_>> {
	let cx = CommandContext::new(
		self_.get_ctx().clone(),
		self_.get_command_interaction().clone(),
	);

	let lang_id = cx.lang_id().await;

	let shard_count = cx.ctx.cache.shard_count();
	let shard = cx.ctx.shard_id.to_string();
	let user_count = UserColor::find().count(&*cx.db).await?;
	let bot = cx.ctx.http.get_current_application_info().await?;

	let bot_name = bot.name.to_string();
	let bot_id = bot.id.to_string();
	let creation_date = format!("<t:{}:F>", bot.id.created_at().unix_timestamp());

	let server_count = cx.ctx.cache.guild_count();
	let app_guild_count = bot.approximate_guild_count.unwrap_or_default() as usize;

	let guild_count = if server_count > app_guild_count {
		app_guild_count
	} else {
		server_count
	};

	let app_installation_count = bot.approximate_user_install_count.unwrap_or_default() as usize;

	let bot_icon = bot.icon.ok_or(anyhow!("No bot icon"))?;

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

	let lib = LIBRARY.to_string();

	let title = USABLE_LOCALES.lookup(&lang_id, "bot_info-title");
	let embed_content = EmbedContent::new(title)
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

	let embed_contents = EmbedsContents::new(vec![embed_content]);

	Ok(embed_contents)
}
