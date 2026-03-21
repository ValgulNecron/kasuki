use anyhow::anyhow;

use crate::command::context::CommandContext;
use crate::command::embed_content::{EmbedContent, EmbedsContents};
use kasuki_macros::slash_command;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use serenity::nonmax::NonMaxU64;
use shared::localization::{Loader, USABLE_LOCALES};

#[slash_command(
	name = "guild", desc = "Get info of the guild.",
	command_type = SubCommand(parent = "server"),
	contexts = [Guild, PrivateChannel],
	install_contexts = [Guild],
)]
async fn guild_command(self_: GuildCommand) -> Result<EmbedsContents<'_>> {
	let cx = CommandContext::new(
		self_.get_ctx().clone(),
		self_.get_command_interaction().clone(),
	);

	let lang_id = cx.lang_id().await;

	let guild_id = cx
		.command_interaction
		.guild_id
		.ok_or(anyhow!("No guild ID"))?;

	let guild = guild_id.to_partial_guild_with_counts(&cx.ctx.http).await?;

	let channels = guild
		.id
		.channels(&cx.ctx.http)
		.await
		.unwrap_or_default()
		.len();

	let guild_id = guild.id;

	let guild_name = guild.name.clone();

	let max_member = guild.max_members.unwrap_or_default();

	let actual_member = guild.approximate_member_count.unwrap_or_default();

	let online_member = guild.approximate_presence_count.unwrap_or_default();

	let max_online = guild
		.max_presences
		.unwrap_or(NonMaxU64::new(25000).unwrap_or_default());

	let guild_banner = guild.banner_url();

	let guild_avatar = guild.icon_url();

	let guild_lang = guild.preferred_locale;

	let guild_premium = guild.premium_tier;

	let guild_sub = guild.premium_subscription_count.unwrap_or_default();

	let guild_nsfw = guild.nsfw_level;

	let creation_date = format!("<t:{}:F>", guild.id.created_at().unix_timestamp());

	let owner = guild
		.owner_id
		.to_user(&cx.ctx.http)
		.await
		.map(|u| u.tag().to_string())
		.unwrap_or_default();

	let roles = guild.roles.len();

	let verification_level = guild.verification_level;

	let mut fields: Vec<(String, String, bool)> = Vec::new();

	fields.push((
		USABLE_LOCALES.lookup(&lang_id, "server_guild-guild_id"),
		guild_id.to_string(),
		true,
	));

	fields.push((
		USABLE_LOCALES.lookup(&lang_id, "server_guild-guild_name"),
		guild_name.to_string(),
		true,
	));

	fields.push((
		USABLE_LOCALES.lookup(&lang_id, "server_guild-member"),
		format!("{}/{}", actual_member, max_member),
		true,
	));

	fields.push((
		USABLE_LOCALES.lookup(&lang_id, "server_guild-online"),
		format!("{}/{}", online_member, max_online),
		true,
	));

	fields.push((
		USABLE_LOCALES.lookup(&lang_id, "server_guild-creation_date"),
		creation_date,
		true,
	));

	fields.push((
		USABLE_LOCALES.lookup(&lang_id, "server_guild-lang"),
		guild_lang.to_string(),
		true,
	));

	fields.push((
		USABLE_LOCALES.lookup(&lang_id, "server_guild-premium"),
		format!("{:?}", guild_premium),
		true,
	));

	fields.push((
		USABLE_LOCALES.lookup(&lang_id, "server_guild-sub"),
		guild_sub.to_string(),
		true,
	));

	fields.push((
		USABLE_LOCALES.lookup(&lang_id, "server_guild-nsfw"),
		format!("{:?}", guild_nsfw),
		true,
	));

	fields.push((
		USABLE_LOCALES.lookup(&lang_id, "server_guild-owner"),
		owner.to_string(),
		true,
	));

	fields.push((
		USABLE_LOCALES.lookup(&lang_id, "server_guild-roles"),
		roles.to_string(),
		true,
	));

	fields.push((
		USABLE_LOCALES.lookup(&lang_id, "server_guild-channels"),
		channels.to_string(),
		true,
	));

	fields.push((
		USABLE_LOCALES.lookup(&lang_id, "server_guild-verification_level"),
		format!("{:?}", verification_level),
		true,
	));

	let mut embed_content = EmbedContent::new(String::new()).fields(fields);

	if guild_avatar.is_some() {
		embed_content = embed_content.thumbnail(guild_avatar.unwrap())
	}

	if guild_banner.is_some() {
		embed_content = embed_content.images_url(guild_banner.unwrap())
	}

	let embed_contents = EmbedsContents::new(vec![embed_content]);

	Ok(embed_contents)
}
