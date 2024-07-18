use std::error::Error;
use std::sync::Arc;

use moka::future::Cache;
use serenity::all::{
    CommandInteraction, Context, CreateInteractionResponse, CreateInteractionResponseMessage,
};
use tokio::sync::RwLock;

use crate::config::Config;
use crate::helper::create_default_embed::get_default_embed;
use crate::helper::error_management::error_enum::ResponseError;
use crate::helper::get_option::subcommand::get_option_map_string_subcommand;
use crate::helper::vndbapi::user::get_user;
use crate::structure::message::vn::user::load_localization_user;
use crate::structure::message::vn::user::UserLocalised;

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
    let user = map
        .get(&String::from("username"))
        .ok_or(ResponseError::Option(String::from("No username provided")))?;
    let path = format!("/user?q={}&fields=lengthvotes,lengthvotes_sum", user);
    let user = get_user(path, vndb_cache).await?;
    let user_localised: UserLocalised = load_localization_user(guild_id, db_type).await?;
    let fields = vec![
        (user_localised.id.clone(), user.id.clone(), true),
        (
            user_localised.playtime.clone(),
            user.lengthvotes.to_string(),
            true,
        ),
        (
            user_localised.playtimesum.clone(),
            user.lengthvotes_sum.to_string(),
            true,
        ),
        (user_localised.name.clone(), user.username.clone(), true),
    ];
    let builder_embed = get_default_embed(None)
        .title(user_localised.title)
        .fields(fields);
    let builder_message = CreateInteractionResponseMessage::new().embed(builder_embed);
    let builder = CreateInteractionResponse::Message(builder_message);
    command_interaction
        .create_response(&ctx.http, builder)
        .await
        .map_err(|e| ResponseError::Sending(format!("{:#?}", e)))?;
    Ok(())
}
