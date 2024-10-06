use std::error::Error;
use std::sync::Arc;

use crate::command::command_trait::{Command, SlashCommand};
use crate::config::Config;
use crate::helper::create_default_embed::get_default_embed;
use crate::helper::get_option::subcommand::get_option_map_string_subcommand;
use crate::helper::vndbapi::producer::get_producer;
use crate::structure::message::vn::producer::load_localization_producer;
use markdown_converter::vndb::convert_vndb_markdown;
use moka::future::Cache;
use serenity::all::{
    CommandInteraction, Context, CreateInteractionResponse, CreateInteractionResponseMessage,
};
use tokio::sync::RwLock;
use tracing::trace;

pub struct VnProducerCommand {
    pub ctx: Context,
    pub command_interaction: CommandInteraction,
    pub config: Arc<Config>,
    pub vndb_cache: Arc<RwLock<Cache<String, String>>>,
}

impl Command for VnProducerCommand {
    fn get_ctx(&self) -> &Context {
        &self.ctx
    }

    fn get_command_interaction(&self) -> &CommandInteraction {
        &self.command_interaction
    }
}

impl SlashCommand for VnProducerCommand {
    async fn run_slash(&self) -> Result<(), Box<dyn Error>> {
        send_embed(
            &self.ctx,
            &self.command_interaction,
            self.config.clone(),
            self.vndb_cache.clone(),
        )
        .await
    }
}

async fn send_embed(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    config: Arc<Config>,
    vndb_cache: Arc<RwLock<Cache<String, String>>>,
) -> Result<(), Box<dyn Error>> {
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };

    let map = get_option_map_string_subcommand(command_interaction);

    trace!("{:?}", map);

    let producer = map
        .get(&String::from("name"))
        .cloned()
        .unwrap_or(String::new());

    let producer_localised = load_localization_producer(guild_id, config.db.clone()).await?;

    let producer = get_producer(producer.clone(), vndb_cache).await?;

    let producer = producer.results[0].clone();

    let mut fields = vec![];

    if let Some(lang) = producer.lang {
        fields.push((producer_localised.lang.clone(), lang, true));
    }

    if let Some(aliases) = producer.aliases {
        let aliases = aliases
            .into_iter()
            .take(10)
            .collect::<Vec<String>>()
            .join(", ");

        fields.push((producer_localised.aliases.clone(), aliases, true));
    }

    if let Some(results_type) = producer.results_type {
        fields.push((
            producer_localised.prod_type.clone(),
            results_type.to_string(),
            true,
        ));
    }

    let builder_embed = get_default_embed(None)
        .description(convert_vndb_markdown(
            &producer.description.unwrap_or_default().clone(),
        ))
        .fields(fields)
        .title(producer.name.clone())
        .url(format!("https://vndb.org/{}", producer.id));

    let builder_message = CreateInteractionResponseMessage::new().embed(builder_embed);

    let builder = CreateInteractionResponse::Message(builder_message);

    command_interaction
        .create_response(&ctx.http, builder)
        .await?;

    Ok(())
}
