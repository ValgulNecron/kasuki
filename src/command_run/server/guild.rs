use serenity::all::{
    CommandInteraction, Context, CreateEmbed, CreateInteractionResponse,
    CreateInteractionResponseMessage, Timestamp,
};

use crate::constant::COLOR;
use crate::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
        use crate::lang_struct::server::guild::load_localization_guild;

pub async fn run(ctx: &Context, command_interaction: &CommandInteraction) -> Result<(), AppError> {
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };

    let guild_localised = load_localization_guild(guild_id).await?;

    let guild_id = command_interaction.guild_id.ok_or(AppError::new(
        String::from("There is no option"),
        ErrorType::Option,
        ErrorResponseType::Message,
    ))?;

    let guild = guild_id
        .to_partial_guild_with_counts(&ctx.http)
        .await
        .map_err(|e| {
            AppError::new(
                format!("Could not get the guild. {}", e),
                ErrorType::Option,
                ErrorResponseType::Message,
            )
        })?;

    let guild_id = guild.id;
    let guild_name = guild.name.clone();
    let max_member = guild.max_members.unwrap_or_default();
    let actual_member = guild.approximate_member_count.unwrap_or_default();
    let online_member = guild.approximate_presence_count.unwrap_or_default();
    let max_online = guild.max_presences.unwrap_or_default();
    let guild_banner = guild.banner_url();
    let guild_avatar = guild.icon_url();
    let guild_lang = guild.preferred_locale;
    let guild_premium = guild.premium_tier;
    let guild_sub = guild.premium_subscription_count.unwrap_or_default();
    let guild_nsfw = guild.nsfw_level;
    let creation_date = format!("<t:{}:F>", guild.id.created_at().unix_timestamp());
    let mut fields:Vec<(String, String, bool)> = Vec::new();

    fields.push((guild_localised.guild_id, guild_id.to_string(), true));
    fields.push((guild_localised.guild_name, guild_name, true));
    fields.push((guild_localised.member, format!("{}/{}", actual_member, max_member), true));
    fields.push((guild_localised.online, format!("{}/{}", online_member, max_online), true));
    fields.push((guild_localised.creation_date, creation_date, true));
    fields.push((guild_localised.lang, guild_lang, true));
    fields.push((guild_localised.premium, format!("{:?}", guild_premium), true));
    fields.push((guild_localised.sub, guild_sub.to_string(), true));
    fields.push((guild_localised.nsfw, format!("{:?}", guild_nsfw), true));

    let mut builder_embed = CreateEmbed::new()
        .timestamp(Timestamp::now())
        .color(COLOR)
        .fields(fields);

    if guild_avatar.is_some() {
        builder_embed = builder_embed.thumbnail(guild_avatar.unwrap())
    }

    if guild_banner.is_some() {
        builder_embed = builder_embed.image(guild_banner.unwrap())
    }

    let builder_message = CreateInteractionResponseMessage::new().embed(builder_embed);

    let builder = CreateInteractionResponse::Message(builder_message);

    command_interaction
        .create_response(&ctx.http, builder)
        .await
        .map_err(|e| {
            AppError::new(
                format!("Error while sending the command {}", e),
                ErrorType::Command,
                ErrorResponseType::Message,
            )
        })
}
