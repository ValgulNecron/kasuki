use crate::command::command_trait::{Command, SlashCommand};
use crate::database::guild_lang;
use crate::database::prelude::GuildLang;
use crate::event_handler::BotData;
use crate::get_url;
use crate::helper::create_default_embed::get_default_embed;
use crate::helper::get_option::subcommand_group::get_option_map_string_subcommand_group;
use crate::structure::message::admin::server::lang::load_localization_lang;
use anyhow::{anyhow, Result};
use sea_orm::ActiveValue::Set;
use sea_orm::EntityTrait;
use serenity::all::{
    CommandInteraction, Context as SerenityContext, CreateInteractionResponse,
    CreateInteractionResponseMessage,
};

pub struct LangCommand {
    pub ctx: SerenityContext,
    pub command_interaction: CommandInteraction,
}

impl Command for LangCommand {
    fn get_ctx(&self) -> &SerenityContext {
        &self.ctx
    }

    fn get_command_interaction(&self) -> &CommandInteraction {
        &self.command_interaction
    }
}

impl SlashCommand for LangCommand {
    async fn run_slash(&self) -> Result<()> {
        send_embed(&self.ctx, &self.command_interaction).await
    }
}

async fn send_embed(ctx: &SerenityContext, command_interaction: &CommandInteraction) -> Result<()> {
    let map = get_option_map_string_subcommand_group(command_interaction);
    let bot_data = ctx.data::<BotData>().clone();

    let lang = map
        .get(&String::from("lang_choice"))
        .ok_or(anyhow!("No option for lang_choice"))?;

    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };

    let connection = sea_orm::Database::connect(get_url(bot_data.config.db.clone())).await?;

    GuildLang::insert(guild_lang::ActiveModel {
        guild_id: Set(guild_id.clone()),
        lang: Set(lang.clone()),
        ..Default::default()
    })
    .exec(&connection)
    .await?;

    let lang_localised = load_localization_lang(guild_id, bot_data.config.db.clone()).await?;

    let builder_embed = get_default_embed(None)
        .description(lang_localised.desc.replace("$lang$", lang.as_str()))
        .title(&lang_localised.title);

    let builder_message = CreateInteractionResponseMessage::new().embed(builder_embed);

    let builder = CreateInteractionResponse::Message(builder_message);

    command_interaction
        .create_response(&ctx.http, builder)
        .await?;

    Ok(())
}
