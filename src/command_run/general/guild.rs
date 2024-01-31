use serenity::all::{
    CommandInteraction, Context, CreateEmbed, CreateInteractionResponse,
    CreateInteractionResponseMessage, Timestamp,
};

use crate::constant::COLOR;
use crate::error_enum::AppError;
use crate::error_enum::AppError::{CommandSendingError, OptionError};
use crate::lang_struct::general::guild::load_localization_guild;

pub async fn run(ctx: &Context, command_interaction: &CommandInteraction) -> Result<(), AppError> {
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };

    let guild_localised = load_localization_guild(guild_id).await?;

    let guild_id = command_interaction
        .guild_id
        .ok_or(OptionError(String::from("There is no option")))?;

    let guild = guild_id
        .to_partial_guild_with_counts(&ctx.http)
        .await
        .map_err(|e| OptionError(format!("There is no option {}", e)))?;

    let guild_name = guild.name.clone();
    let created_date = guild
        .id
        .created_at()
        .clone()
        .format("%Y-%m-%d %H:%M:%S")
        .to_string();
    let number_of_member = guild.approximate_member_count.unwrap_or(0).to_string();
    let max_member = match guild.max_members {
        Some(max) => max.to_string(),
        None => String::from("Unknown"),
    };

    let desc = guild_localised
        .desc
        .replace("$name$", guild_name.as_str())
        .replace("$date$", created_date.as_str())
        .replace("$number$", number_of_member.as_str())
        .replace("$max$", max_member.as_str());

    let mut builder_embed = CreateEmbed::new()
        .timestamp(Timestamp::now())
        .color(COLOR)
        .description(desc)
        .title(&guild_localised.title);

    match guild.icon_hash {
        Some(hash) => {
            let icon_url = format!("https://cdn.discordapp.com/icons/{}/{}.png", guild.id, hash);
            builder_embed = builder_embed.thumbnail(icon_url)
        }
        None => {
            if let Some(hash) = guild.icon {
                let icon_url =
                    format!("https://cdn.discordapp.com/icons/{}/{}.png", guild.id, hash);
                builder_embed = builder_embed.thumbnail(icon_url)
            }
        }
    };

    if let Some(banner) = guild.banner {
        builder_embed = builder_embed.image(banner)
    }

    let builder_message = CreateInteractionResponseMessage::new().embed(builder_embed);

    let builder = CreateInteractionResponse::Message(builder_message);

    command_interaction
        .create_response(&ctx.http, builder)
        .await
        .map_err(|e| CommandSendingError(format!("Error while sending the command {}", e)))
}
