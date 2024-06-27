use crate::config::Config;
use crate::helper::create_default_embed::get_default_embed;
use crate::helper::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use crate::helper::get_option::subcommand::get_option_map_string_subcommand;
use crate::helper::vndbapi::staff::get_staff;
use crate::structure::message::vn::staff::load_localization_staff;
use markdown_converter::vndb::convert_vndb_markdown;
use serenity::all::{
    CommandInteraction, Context, CreateInteractionResponse, CreateInteractionResponseMessage,
};
use std::sync::Arc;
use tracing::trace;

pub async fn run(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    config: Arc<Config>,
) -> Result<(), AppError> {
    let db_type = config.bot.config.db_type.clone();
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };
    let map = get_option_map_string_subcommand(command_interaction);
    trace!("{:?}", map);
    let staff = map
        .get(&String::from("name"))
        .cloned()
        .unwrap_or(String::new());
    let staff_localised = load_localization_staff(guild_id, db_type).await?;

    let staff = get_staff(staff.clone()).await?;
    let staff = staff.results[0].clone();
    let mut fields = vec![];

    fields.push((staff_localised.lang.clone(), staff.lang, true));
    fields.push((staff_localised.aid.clone(), staff.aid.to_string(), true));
    fields.push((staff_localised.gender.clone(), staff.gender.clone(), true));
    fields.push((staff_localised.main.clone(), staff.ismain.to_string(), true));

    let builder_embed = get_default_embed(None)
        .description(convert_vndb_markdown(&staff.description.clone()))
        .fields(fields)
        .title(staff.name.clone())
        .url(format!("https://vndb.org/{}", staff.id));
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
