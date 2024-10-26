use crate::config::DbConfig;
use crate::constant::{ACTIVITY_LIST_LIMIT, COLOR};
use crate::database::activity_data::{Column, Model};
use crate::database::prelude::ActivityData;
use crate::get_url;
use crate::structure::message::anilist_server::list_all_activity::load_localization_list_activity;
use anyhow::{anyhow, Result};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serenity::all::{
    ComponentInteraction, Context as SerenityContext, CreateButton, CreateEmbed,
    CreateInteractionResponse, CreateInteractionResponseMessage, Timestamp,
};
use tracing::trace;

pub async fn update(
    ctx: &SerenityContext,
    component_interaction: &ComponentInteraction,
    page_number: &str,
    db_config: DbConfig,
) -> Result<()> {
    let guild_id = match component_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };

    let list_activity_localised_text =
        load_localization_list_activity(guild_id, db_config.clone()).await?;

    let guild_id = component_interaction
        .guild_id
        .ok_or(anyhow!("Guild ID not found"))?;

    let connection = sea_orm::Database::connect(get_url(db_config.clone())).await?;

    let list = ActivityData::find()
        .filter(Column::ServerId.eq(guild_id.to_string()))
        .all(&connection)
        .await?;

    let len = list.len();

    let actual_page: u64 = page_number.parse()?;

    trace!("{:?}", actual_page);

    let next_page: u64 = actual_page + 1;

    let previous_page: u64 = if actual_page > 0 { actual_page - 1 } else { 0 };

    let activity: Vec<String> = get_formatted_activity_list(list, actual_page);

    let join_activity = activity.join("\n");

    let builder_message = CreateEmbed::new()
        .timestamp(Timestamp::now())
        .color(COLOR)
        .title(list_activity_localised_text.title)
        .description(join_activity);

    let mut message_rep = CreateInteractionResponseMessage::new().embed(builder_message);

    if page_number != "0" {
        message_rep = message_rep.button(
            CreateButton::new(format!("next_activity_{}", previous_page))
                .label(&list_activity_localised_text.previous),
        );
    }

    trace!("{:?}", len);

    trace!("{:?}", ACTIVITY_LIST_LIMIT);

    if len > ACTIVITY_LIST_LIMIT as usize
        && (len > (ACTIVITY_LIST_LIMIT * (actual_page + 1)) as usize)
    {
        message_rep = message_rep.button(
            CreateButton::new(format!("next_activity_{}", next_page))
                .label(&list_activity_localised_text.next),
        )
    }

    let response = CreateInteractionResponse::UpdateMessage(message_rep);

    component_interaction
        .create_response(&ctx.http, response)
        .await?;

    Ok(())
}

pub fn get_formatted_activity_list(list: Vec<Model>, actual_page: u64) -> Vec<String> {
    list.into_iter()
        .map(|activity| {
            let anime_id = activity.anime_id;

            let name = activity.name;

            format!("[{}](https://anilist_user.co/anime/{})", name, anime_id)
        })
        .skip((ACTIVITY_LIST_LIMIT * actual_page) as usize)
        .take(ACTIVITY_LIST_LIMIT as usize)
        .collect()
}
