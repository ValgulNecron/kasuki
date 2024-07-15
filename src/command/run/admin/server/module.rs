use std::error::Error;
use std::sync::Arc;

use serenity::all::{
    CommandInteraction, Context, CreateInteractionResponse, CreateInteractionResponseMessage,
};

use crate::config::Config;
use crate::database::data_struct::module_status::ActivationStatusModule;
use crate::database::manage::dispatcher::data_dispatch::{
    get_data_module_activation_status, set_data_module_activation_status,
};
use crate::helper::create_default_embed::get_default_embed;
use crate::helper::error_management::error_enum::ResponseError;
use crate::helper::get_option::subcommand_group::{
    get_option_map_boolean_subcommand_group, get_option_map_string_subcommand_group,
};
use crate::structure::message::admin::server::module::load_localization_module_activation;

/// This asynchronous function runs the command interaction for setting the activation status of a module.
///
/// It first retrieves the guild ID from the command interaction. If the command interaction does not have a guild ID, it uses "0" as the guild ID.
///
/// It retrieves the module name and state from the command interaction options. If either option is not found, it returns an `AppError`.
///
/// It retrieves the localized module activation data for the guild.
///
/// It retrieves the current module activation status from the database.
///
/// It sets the new module activation status based on the module name and state retrieved from the command interaction options.
/// If the module name does not match any of the known modules, it returns an `AppError` indicating that the module does not exist.
///
/// It sets the new module activation status in the database.
///
/// It creates an embed for the response message, including the module name and a description indicating whether the module is on or off.
///
/// It creates a response message with the embed.
///
/// It sends the response to the command interaction. If an error occurs during this process, it returns an `AppError` indicating that there was an error while sending the command.
///
/// # Arguments
///
/// * `ctx` - The context in which this function is being called.
/// * `command_interaction` - The command interaction that triggered this function.
///
/// # Returns
///
/// A `Result` indicating whether the function executed successfully. If an error occurred, it contains an `AppError`.
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
    let map = get_option_map_string_subcommand_group(command_interaction);
    let module = map
        .get(&String::from("name"))
        .ok_or(ResponseError::Option(String::from("No option for name")))?;
    let module_localised =
        load_localization_module_activation(guild_id.clone(), db_type.clone()).await?;
    let map = get_option_map_boolean_subcommand_group(command_interaction);
    let state = *map
        .get(&String::from("state"))
        .ok_or(ResponseError::Option(String::from("No option for state")))?;

    let row = get_data_module_activation_status(&guild_id, db_type.clone()).await?;
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

    set_data_module_activation_status(module_status, db_type).await?;
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

/// This asynchronous function checks the activation status of a module.
///
/// It checks the activation status of the module based on the module name. If the module name does not match any of the known modules, it returns false.
///
/// # Arguments
///
/// * `module` - The name of the module to check.
/// * `row` - The current module activation status.
///
/// # Returns
///
/// A boolean indicating whether the module is activated.
pub async fn check_activation_status(module: &str, row: ActivationStatusModule) -> bool {
    match module {
        "ANILIST" => row.anilist_module.unwrap_or(true),
        "AI" => row.ai_module.unwrap_or(true),
        "GAME" => row.game_module.unwrap_or(true),
        "NEW_MEMBER" => row.new_member.unwrap_or(false),
        "ANIME" => row.anime.unwrap_or(true),
        "VN" => row.vn.unwrap_or(true),
        _ => false,
    }
}
