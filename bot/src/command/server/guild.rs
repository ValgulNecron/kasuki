use anyhow::{anyhow, Result};

use crate::command::command_trait::{Command, Embed, EmbedContent, SlashCommand};
use crate::event_handler::BotData;
use crate::structure::message::server::guild::load_localization_guild;
use serenity::all::{
	CommandInteraction, Context as SerenityContext
	,
};
use serenity::nonmax::NonMaxU64;

pub struct GuildCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl Command for GuildCommand {
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}
}

impl SlashCommand for GuildCommand {
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

		// Load the localized guild information
		let guild_localised = load_localization_guild(guild_id, config.db.clone()).await?;

		// Retrieve the guild ID from the command interaction or return an error if it does not exist
		let guild_id = command_interaction.guild_id.ok_or(anyhow!("No guild ID"))?;

		// Retrieve the guild's information or return an error if it could not be retrieved
		let guild = guild_id.to_partial_guild_with_counts(&ctx.http).await?;

		// Retrieve various details about the guild
		let channels = guild.id.channels(&ctx.http).await.unwrap_or_default().len();

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
			.to_user(&ctx.http)
			.await
			.map(|u| u.tag())
			.unwrap_or_default();

		let roles = guild.roles.len();

		let verification_level = guild.verification_level;

		// Initialize a vector to store the fields for the embed
		let mut fields: Vec<(String, String, bool)> = Vec::new();

		// Add the fields to the vector
		fields.push((guild_localised.guild_id, guild_id.to_string(), true));

		fields.push((guild_localised.guild_name, guild_name.to_string(), true));

		fields.push((
			guild_localised.member,
			format!("{}/{}", actual_member, max_member),
			true,
		));

		fields.push((
			guild_localised.online,
			format!("{}/{}", online_member, max_online),
			true,
		));

		fields.push((guild_localised.creation_date, creation_date, true));

		fields.push((guild_localised.lang, guild_lang.to_string(), true));

		fields.push((
			guild_localised.premium,
			format!("{:?}", guild_premium),
			true,
		));

		fields.push((guild_localised.sub, guild_sub.to_string(), true));

		fields.push((guild_localised.nsfw, format!("{:?}", guild_nsfw), true));

		fields.push((guild_localised.owner, owner, true));

		fields.push((guild_localised.roles, roles.to_string(), true));

		fields.push((guild_localised.channels, channels.to_string(), true));

		fields.push((
			guild_localised.verification_level,
			format!("{:?}", verification_level),
			true,
		));

		// Construct the embed for the response

		let mut content = EmbedContent::new(String::new()).fields(fields);

		// Add the guild's avatar to the embed if it exists
		if guild_avatar.is_some() {
			content = content.thumbnail(Some(guild_avatar.unwrap()))
		}

		// Add the guild's banner to the embed if it exists
		if guild_banner.is_some() {
			content = content.images_url(Some(guild_banner.unwrap()))
		}

		self.send_embed(vec![content]).await
	}
}
