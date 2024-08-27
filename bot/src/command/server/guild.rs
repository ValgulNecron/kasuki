use std::error::Error;
use std::sync::Arc;

use crate::command::command_trait::{Command, SlashCommand};
use crate::config::Config;
use crate::helper::create_default_embed::get_default_embed;
use crate::helper::error_management::error_dispatch;
use crate::structure::message::server::guild::load_localization_guild;
use serenity::all::{
    CommandInteraction, Context, CreateInteractionResponse, CreateInteractionResponseMessage,
};

pub struct GuildCommand {
    pub ctx: Context,
    pub command_interaction: CommandInteraction,
    pub config: Arc<Config>,
}

impl Command for GuildCommand {
    fn get_ctx(&self) -> &Context {
        &self.ctx
    }

    fn get_command_interaction(&self) -> &CommandInteraction {
        &self.command_interaction
    }
}

impl SlashCommand for GuildCommand {
    async fn run_slash(&self) -> Result<(), Box<dyn Error>> {
        send_embed(&self.ctx, &self.command_interaction, self.config.clone()).await
    }
}
async fn send_embed(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    config: Arc<Config>,
) -> Result<(), Box<dyn Error>> {
    // Retrieve the guild ID from the command interaction
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };

    // Load the localized guild information
    let guild_localised = load_localization_guild(guild_id, config.bot.config.clone()).await?;

    // Retrieve the guild ID from the command interaction or return an error if it does not exist
    let guild_id = command_interaction
        .guild_id
        .ok_or(error_dispatch::Error::Option(String::from("No guild ID")))?;

    // Retrieve the guild's information or return an error if it could not be retrieved
    let guild = guild_id.to_partial_guild_with_counts(&ctx.http).await?;

    // Retrieve various details about the guild
    let channels = guild.channels(&ctx.http).await.unwrap_or_default().len();
    let guild_id = guild.id;
    let guild_name = guild.name.clone();
    let max_member = guild.max_members.unwrap_or_default();
    let actual_member = guild.approximate_member_count.unwrap_or_default();
    let online_member = guild.approximate_presence_count.unwrap_or_default();
    let max_online = guild.max_presences.unwrap_or(25000);
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
    fields.push((guild_localised.guild_name, guild_name, true));
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
    fields.push((guild_localised.lang, guild_lang, true));
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
    let mut builder_embed = get_default_embed(None).fields(fields);

    // Add the guild's avatar to the embed if it exists
    if guild_avatar.is_some() {
        builder_embed = builder_embed.thumbnail(guild_avatar.unwrap())
    }

    // Add the guild's banner to the embed if it exists
    if guild_banner.is_some() {
        builder_embed = builder_embed.image(guild_banner.unwrap())
    }

    // Construct the message for the response
    let builder_message = CreateInteractionResponseMessage::new().embed(builder_embed);

    // Construct the response
    let builder = CreateInteractionResponse::Message(builder_message);

    // Send the response to the command interaction
    command_interaction
        .create_response(&ctx.http, builder)
        .await?;
    Ok(())
}
