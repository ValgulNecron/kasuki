use std::error::Error;
use std::sync::Arc;

use crate::command::command_trait::{Command, SlashCommand};
use crate::config::Config;
use crate::helper::create_default_embed::get_default_embed;
use crate::helper::get_option::subcommand::get_option_map_string_subcommand;
use crate::helper::vndbapi::character::get_character;
use crate::structure::message::vn::character::load_localization_character;
use markdown_converter::vndb::convert_vndb_markdown;
use moka::future::Cache;
use serenity::all::{
    CommandInteraction, Context, CreateInteractionResponse, CreateInteractionResponseMessage,
};
use tokio::sync::RwLock;
use tracing::trace;

pub struct VnCharacterCommand {
    pub ctx: Context,
    pub command_interaction: CommandInteraction,
    pub config: Arc<Config>,
    pub vndb_cache: Arc<RwLock<Cache<String, String>>>,
}

impl Command for VnCharacterCommand {
    fn get_ctx(&self) -> &Context {
        &self.ctx
    }

    fn get_command_interaction(&self) -> &CommandInteraction {
        &self.command_interaction
    }
}

impl SlashCommand for VnCharacterCommand {
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

    let character = map
        .get(&String::from("name"))
        .cloned()
        .unwrap_or(String::new());

    let character_localised = load_localization_character(guild_id, config.db.clone()).await?;

    let character = get_character(character.clone(), vndb_cache).await?;

    let character = character.results[0].clone();

    let mut fields = vec![];

    if let Some(blood_type) = character.blood_type {
        fields.push((character_localised.blood_type.clone(), blood_type, true));
    }

    if let Some(height) = character.height {
        let cm = format!("{}cm", height);

        fields.push((character_localised.height.clone(), cm, true));
    }

    if let Some(weight) = character.weight {
        let weight = format!("{}kg", weight);

        fields.push((character_localised.weight.clone(), weight, true));
    }

    if let Some(age) = character.age {
        fields.push((character_localised.age.clone(), age.to_string(), true));
    }

    if let Some(bust) = character.bust {
        let bust = format!("{}cm", bust);

        fields.push((character_localised.bust.clone(), bust, true));
    }

    if let Some(waist) = character.waist {
        let waist = format!("{}cm", waist);

        fields.push((character_localised.waist.clone(), waist, true));
    }

    if let Some(hips) = character.hips {
        let hips = format!("{}cm", hips);

        fields.push((character_localised.hip.clone(), hips, true));
    }

    if let Some(cup) = character.cup {
        fields.push((character_localised.cup.clone(), cup, true));
    }

    let sex = format!("{}, ||{}||", character.sex[0], character.sex[1]);

    fields.push((character_localised.sex, sex, true));

    if let Some(birthday) = character.birthday {
        let birthday = format!("{:02}/{:02}", birthday[0], birthday[1]);

        fields.push((character_localised.birthday.clone(), birthday, true));
    }

    let vns = character
        .vns
        .iter()
        .map(|vn| vn.title.clone())
        .take(10)
        .collect::<Vec<String>>()
        .join(", ");

    fields.push((character_localised.vns.clone(), vns, true));

    let traits = character
        .traits
        .iter()
        .map(|traits| traits.name.clone())
        .take(10)
        .collect::<Vec<String>>()
        .join(", ");

    fields.push((character_localised.traits.clone(), traits, true));

    let mut builder_embed = get_default_embed(None)
        .description(convert_vndb_markdown(
            &character.description.unwrap_or_default().clone(),
        ))
        .fields(fields)
        .title(character.name.clone())
        .url(format!("https://vndb.org/{}", character.id));

    let sexual = match character.image.clone() {
        Some(image) => image.sexual,
        None => 2.0,
    };

    let violence = match character.image.clone() {
        Some(image) => image.violence,
        None => 2.0,
    };

    let url: Option<String> = match character.image {
        Some(image) => Some(image.url.clone()),
        None => None,
    };

    if (sexual <= 1.5) && (violence <= 1.0) {
        if let Some(url) = url {
            builder_embed = builder_embed.image(url);
        }
    }

    let builder_message = CreateInteractionResponseMessage::new().embed(builder_embed);

    let builder = CreateInteractionResponse::Message(builder_message);

    command_interaction
        .create_response(&ctx.http, builder)
        .await?;

    Ok(())
}
