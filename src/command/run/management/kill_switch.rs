use std::error::Error;
use std::sync::Arc;

use serenity::all::{
    CommandInteraction, Context, CreateInteractionResponse, CreateInteractionResponseMessage,
};

use crate::config::Config;
use crate::database::data_struct::module_status::ActivationStatusModule;
use crate::database::manage::dispatcher::data_dispatch::{
    get_data_module_activation_kill_switch_status, set_data_kill_switch_activation_status,
};
use crate::helper::create_default_embed::get_default_embed;
use crate::helper::error_management::error_enum::ResponseError;
use crate::helper::get_option::command::{get_option_map_boolean, get_option_map_string};
use crate::structure::message::admin::server::module::load_localization_module_activation;
use crate::structure::message::management::kill_switch::load_localization_kill_switch;

pub async fn run(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    config: Arc<Config>,
) -> Result<(), Box<dyn Error>> {
    let db_type = config.bot.config.db_type.clone();
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };
    let map = get_option_map_string(command_interaction);
    let module = map
        .get(&String::from("name"))
        .ok_or(ResponseError::Option(String::from("No option for name")))?;
    let module_localised = load_localization_kill_switch(guild_id.clone(), db_type.clone()).await?;
    let map = get_option_map_boolean(command_interaction);
    let state = *map
        .get(&String::from("state"))
        .ok_or(ResponseError::Option(String::from("No option for state")))?;

    let row = get_data_module_activation_kill_switch_status(db_type.clone()).await?;

    let mut ai_value = row.ai_module.unwrap_or(true);
    let mut anilist_value = row.anilist_module.unwrap_or(true);
    let mut game_value = row.game_module.unwrap_or(true);
    let mut new_member_value = row.new_member.unwrap_or(false);
    let mut anime_value = row.anime.unwrap_or(true);
    let mut vn_value = row.vn.unwrap_or(true);
    match module.as_str() {
        "ANILIST" => anilist_value = state,
        "AI" => ai_value = state,
        "GAME" => game_value = state,
        "NEW_MEMBER" => new_member_value = state,
        "ANIME" => anime_value = state,
        "VN" => vn_value = state,
        _ => {
            return Err(Box::new(ResponseError::Option(String::from(
                "The module specified does not exist",
            ))));
        }
    }

    let module_status = ActivationStatusModule {
        guild_id: Some(guild_id),
        ai_module: Some(ai_value),
        anilist_module: Some(anilist_value),
        game_module: Some(game_value),
        new_member: Some(new_member_value),
        anime: Some(anime_value),
        vn: Some(vn_value),
    };

    set_data_kill_switch_activation_status(module_status, db_type).await?;
    let desc = if state {
        &module_localised.on
    } else {
        &module_localised.off
    };
    let builder_embed = get_default_embed(None).description(desc).title(module);

    let builder_message = CreateInteractionResponseMessage::new().embed(builder_embed);

    let builder = CreateInteractionResponse::Message(builder_message);

    command_interaction
        .create_response(&ctx.http, builder)
        .await
        .map_err(|e| ResponseError::Sending(format!("{:#?}", e)))?;
    Ok(())
}
