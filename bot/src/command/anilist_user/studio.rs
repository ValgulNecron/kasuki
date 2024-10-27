use crate::command::command_trait::Command;
use anyhow::{anyhow, Result};
use std::sync::Arc;

use crate::command::command_trait::SlashCommand;
use crate::config::Config;
use crate::constant::DEFAULT_STRING;
use crate::event_handler::BotData;
use crate::helper::create_default_embed::get_default_embed;
use crate::helper::get_option::command::get_option_map_string;
use crate::helper::make_graphql_cached::make_request_anilist;
use crate::structure::message::anilist_user::studio::load_localization_studio;
use crate::structure::run::anilist::studio::{
    StudioQuerryId, StudioQuerryIdVariables, StudioQuerrySearch, StudioQuerrySearchVariables,
};
use cynic::{GraphQlResponse, QueryBuilder};
use moka::future::Cache;
use serenity::all::{
    CommandInteraction, Context as SerenityContext, CreateInteractionResponse,
    CreateInteractionResponseMessage,
};
use small_fixed_array::FixedString;
use tokio::sync::RwLock;

pub struct StudioCommand {
    pub ctx: SerenityContext,
    pub command_interaction: CommandInteraction,
}

impl Command for StudioCommand {
    fn get_ctx(&self) -> &SerenityContext {
        &self.ctx
    }

    fn get_command_interaction(&self) -> &CommandInteraction {
        &self.command_interaction
    }
}

impl SlashCommand for StudioCommand {
    async fn run_slash(&self) -> Result<()> {
        let ctx = self.get_ctx();
        let bot_data = ctx.data::<BotData>().clone();
        let command_interaction = &self.command_interaction;

        let config = bot_data.config.clone();

        let anilist_cache = bot_data.anilist_cache.clone();

        send_embed(ctx, command_interaction, config, anilist_cache).await
    }
}

async fn send_embed(
    ctx: &SerenityContext,
    command_interaction: &CommandInteraction,
    config: Arc<Config>,
    anilist_cache: Arc<RwLock<Cache<String, String>>>,
) -> Result<()> {
    // Retrieve the name or ID of the studio from the command interaction
    let map = get_option_map_string(command_interaction);

    let value = map
        .get(&FixedString::from_str_trunc("studio"))
        .ok_or(anyhow!("No studio specified"))?;

    // Fetch the studio's data from AniList
    let studio = if value.parse::<i32>().is_ok() {
        let id = value.parse::<i32>()?;

        let var = StudioQuerryIdVariables { id: Some(id) };

        let operation = StudioQuerryId::build(var);

        let data: GraphQlResponse<StudioQuerryId> =
            make_request_anilist(operation, false, anilist_cache).await?;

        data.data.unwrap().studio.unwrap()
    } else {
        let var = StudioQuerrySearchVariables {
            search: Some(value.as_str()),
        };

        let operation = StudioQuerrySearch::build(var);

        let data: GraphQlResponse<StudioQuerrySearch> =
            make_request_anilist(operation, false, anilist_cache).await?;

        data.data.unwrap().studio.unwrap()
    };

    // Retrieve the guild ID from the command interaction
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };

    // Load the localized studio strings
    let studio_localised = load_localization_studio(guild_id, config.db.clone()).await?;

    // Initialize a string to store the content of the response
    let mut content = String::new();

    // Iterate over the nodes of the studio's media
    for m in studio.media.unwrap().nodes.unwrap() {
        // Clone the title of the media
        let m = m.unwrap();

        let title = m.title.unwrap().clone();

        // Retrieve the romaji and user-preferred titles
        let rj = title.romaji.unwrap_or_default();

        let en = title.user_preferred.unwrap_or_default();

        // Format the text for the response
        let text = format!(
            "[{}/{}]({})",
            rj,
            en,
            m.site_url.unwrap_or(DEFAULT_STRING.clone())
        );

        // Append the text to the content string
        content.push_str(text.as_str());

        content.push('\n')
    }

    // Construct the description for the response
    let desc = studio_localised
        .desc
        .replace("$id$", studio.id.to_string().as_str())
        .replace(
            "$fav$",
            studio.favourites.unwrap_or_default().to_string().as_str(),
        )
        .replace(
            "$animation$",
            studio.is_animation_studio.to_string().as_str(),
        )
        .replace("$list$", content.as_str());

    // Retrieve the name of the studio
    let name = studio.name;

    // Construct the embed for the response
    let builder_embed = get_default_embed(None)
        .description(desc)
        .title(name)
        .url(studio.site_url.unwrap_or_default());

    // Construct the message for the response
    let builder_message = CreateInteractionResponseMessage::new().embed(builder_embed);

    // Construct the response
    let builder = CreateInteractionResponse::Message(builder_message);

    // Send the response to the command interaction
    command_interaction
        .create_response(&ctx.http, builder)
        .await?;

    Ok(())
}
