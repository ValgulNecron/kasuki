use crate::command::command_trait::{Command, SlashCommand};
use crate::config::Config;
use crate::get_url;
use crate::helper::create_default_embed::get_default_embed;
use crate::helper::error_management::error_dispatch;
use crate::helper::get_option::subcommand_group::{
    get_option_map_boolean_subcommand_group, get_option_map_string_subcommand_group,
};
use crate::structure::database::module_activation::Model;
use crate::structure::database::prelude::ModuleActivation;
use crate::structure::message::admin::server::module::load_localization_module_activation;
use sea_orm::ColumnTrait;
use sea_orm::{ActiveModelTrait, EntityTrait, IntoActiveModel, QueryFilter};
use serenity::all::{
    CommandInteraction, Context, CreateInteractionResponse, CreateInteractionResponseMessage,
};
use std::error::Error;
use std::sync::Arc;

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
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };
    let map = get_option_map_string_subcommand_group(command_interaction);
    let module = map
        .get(&String::from("name"))
        .ok_or(error_dispatch::Error::Option(String::from(
            "No option for name",
        )))?;
    let module_localised =
        load_localization_module_activation(guild_id.clone(), config.db.clone()).await?;
    let map = get_option_map_boolean_subcommand_group(command_interaction);
    let state = *map
        .get(&String::from("state"))
        .ok_or(error_dispatch::Error::Option(String::from(
            "No option for state",
        )))?;
    let connection = sea_orm::Database::connect(get_url(config.db.clone())).await?;
    let mut row = ModuleActivation::find()
        .filter(crate::structure::database::module_activation::Column::GuildId.eq(guild_id.clone()))
        .one(&connection)
        .await?
        .unwrap_or(Model {
            guild_id,
            ai_module: true,
            anilist_module: true,
            game_module: true,
            new_members_module: false,
            anime_module: true,
            vn_module: true,
            updated_at: Default::default(),
        });
    match module.as_str() {
        "ANILIST" => row.anilist_module = state,
        "AI" => row.ai_module = state,
        "GAME" => row.game_module = state,
        "NEW_MEMBER" => row.new_members_module = state,
        "ANIME" => row.anime_module = state,
        "VN" => row.vn_module = state,
        _ => {
            return Err(Box::new(error_dispatch::Error::Option(String::from(
                "The module specified does not exist",
            ))));
        }
    }

    let active_model = row.into_active_model();
    active_model.update(&connection).await?;

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
        .await?;
    Ok(())
}

pub async fn check_activation_status(module: &str, row: Model) -> bool {
    match module {
        "ANILIST" => row.anilist_module,
        "AI" => row.ai_module,
        "GAME" => row.game_module,
        "NEW_MEMBER" => row.new_members_module,
        "ANIME" => row.anime_module,
        "VN" => row.vn_module,
        _ => false,
    }
}
