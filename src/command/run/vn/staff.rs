use std::error::Error;
use std::sync::Arc;

use markdown_converter::vndb::convert_vndb_markdown;
use moka::future::Cache;
use serenity::all::{
    CommandInteraction, Context, CreateInteractionResponse, CreateInteractionResponseMessage,
};
use tokio::sync::RwLock;
use tracing::trace;

use crate::config::Config;
use crate::helper::create_default_embed::get_default_embed;
use crate::helper::error_management::error_enum::ResponseError;
use crate::helper::get_option::subcommand::get_option_map_string_subcommand;
use crate::helper::vndbapi::staff::get_staff;
use crate::structure::message::vn::staff::load_localization_staff;

pub async fn run(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    config: Arc<Config>,
    vndb_cache: Arc<RwLock<Cache<String, String>>>,
) -> Result<(), Box<dyn Error>> {
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
    let staff_localised =
        load_localization_staff(guild_id, db_type, config.bot.config.clone()).await?;

    let staff = get_staff(staff.clone(), vndb_cache).await?;
    let staff = staff.results[0].clone();
    let fields = vec![
        (staff_localised.lang.clone(), staff.lang, true),
        (staff_localised.aid.clone(), staff.aid.to_string(), true),
        (staff_localised.gender.clone(), staff.gender.clone(), true),
        (staff_localised.main.clone(), staff.ismain.to_string(), true),
    ];

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
        .map_err(|e| ResponseError::Sending(format!("{:#?}", e)))?;
    Ok(())
}
