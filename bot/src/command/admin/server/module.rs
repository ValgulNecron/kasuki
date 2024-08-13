use std::error::Error;
use std::sync::Arc;

use crate::command::command_trait::{Command, SlashCommand};
use crate::config::Config;
use crate::structure::database::module_status::ActivationStatusModule;
use crate::database::dispatcher::data_dispatch::{
    get_data_module_activation_status, set_data_module_activation_status,
};
use crate::helper::create_default_embed::get_default_embed;
use crate::helper::error_management::error_enum::ResponseError;
use crate::helper::get_option::subcommand_group::{
    get_option_map_boolean_subcommand_group, get_option_map_string_subcommand_group,
};
use crate::structure::message::admin::server::module::load_localization_module_activation;
use serenity::all::{
    CommandInteraction, Context, CreateInteractionResponse, CreateInteractionResponseMessage,
};

pub struct ModuleCommand {
    pub ctx: Context,
    pub command_interaction: CommandInteraction,
    pub config: Arc<Config>,
}

impl Command for ModuleCommand {
    fn get_ctx(&self) -> &Context {
        &self.ctx
    }

    fn get_command_interaction(&self) -> &CommandInteraction {
        &self.command_interaction
    }
}

impl SlashCommand for ModuleCommand {
    async fn run_slash(&self) -> Result<(), Box<dyn Error>> {
        send_embed(&self.ctx, &self.command_interaction, self.config.clone()).await
    }
}

async fn send_embed(
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
    let module_localised = load_localization_module_activation(
        guild_id.clone(),
        db_type.clone(),
        config.bot.config.clone(),
    )
    .await?;
    let map = get_option_map_boolean_subcommand_group(command_interaction);
    let state = *map
        .get(&String::from("state"))
        .ok_or(ResponseError::Option(String::from("No option for state")))?;

    let row =
        get_data_module_activation_status(&guild_id, db_type.clone(), config.bot.config.clone())
            .await?;
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

    set_data_module_activation_status(module_status, db_type, config.bot.config.clone()).await?;
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
