use crate::config::Config;
use crate::helper::create_default_embed::get_default_embed;
use crate::helper::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use crate::helper::vndbapi::stats::get_stats;
use crate::structure::message::vn::stats::load_localization_stats;
use serenity::all::{
    CommandInteraction, Context, CreateInteractionResponse, CreateInteractionResponseMessage,
};
use std::sync::Arc;

pub async fn run(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    config: Arc<Config>,
) -> Result<(), AppError> {
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };
    let stats_localised = load_localization_stats(guild_id).await?;

    let stats = get_stats().await?;

    let mut fields = vec![];
    fields.push((stats_localised.chars.clone(), stats.chars.to_string(), true));
    fields.push((
        stats_localised.producer.clone(),
        stats.producers.to_string(),
        true,
    ));
    fields.push((
        stats_localised.release.clone(),
        stats.releases.to_string(),
        true,
    ));
    fields.push((stats_localised.staff.clone(), stats.staff.to_string(), true));
    fields.push((stats_localised.tags.clone(), stats.tags.to_string(), true));
    fields.push((
        stats_localised.traits.clone(),
        stats.traits.to_string(),
        true,
    ));
    fields.push((stats_localised.vns.clone(), stats.vn.to_string(), true));
    fields.push((stats_localised.api.clone(), String::from("VNDB API"), true));
    let builder_embed = get_default_embed(None)
        .title(stats_localised.title)
        .fields(fields);
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
