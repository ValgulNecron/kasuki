use crate::constant::{ACTIVITY_LIST_LIMIT, COLOR};
use crate::database::dispatcher::data_dispatch::get_all_server_activity;
use crate::error_enum::AppError;
use crate::error_enum::AppError::{CommandSendingError, OptionError};
use crate::lang_struct::anilist::list_all_activity::load_localization_list_activity;
use serenity::all::{
    ComponentInteraction, Context, CreateButton, CreateEmbed, EditMessage, Timestamp,
};
use tracing::trace;

pub async fn update(
    ctx: &Context,
    component_interaction: &ComponentInteraction,
    page_number: &str,
) -> Result<(), AppError> {
    let guild_id = match component_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };

    let list_activity_localised_text = load_localization_list_activity(guild_id).await?;

    let guild_id = component_interaction
        .guild_id
        .ok_or(OptionError(String::from("There is no option")))?;

    let list = get_all_server_activity(&guild_id.to_string()).await?;
    let len = list.len();
    let actual_page: u64 = page_number.parse().unwrap();
    let next_page: u64 = actual_page + 1;

    let activity: Vec<String> = list
        .into_iter()
        .map(|activity| {
            let anime_id = activity.anime_id;
            let name = activity.name;
            format!(
                "[{}](https://anilist.co/anime/{})",
                name.unwrap_or_default(),
                anime_id.unwrap_or_default()
            )
        })
        .skip((ACTIVITY_LIST_LIMIT * actual_page) as usize)
        .take(ACTIVITY_LIST_LIMIT as usize)
        .collect();

    let join_activity = activity.join("\n");

    let builder_message = CreateEmbed::new()
        .timestamp(Timestamp::now())
        .color(COLOR)
        .title(list_activity_localised_text.title)
        .description(join_activity);

    let mut response = EditMessage::new().embed(builder_message);
    if page_number != "0" {
        response = response.button(
            CreateButton::new(format!("next_user_{}", page_number))
                .label(&list_activity_localised_text.previous),
        );
    }
    trace!("{:?}", len);
    trace!("{:?}", ACTIVITY_LIST_LIMIT);
    if len > ACTIVITY_LIST_LIMIT as usize {
        response = response.button(
            CreateButton::new(format!("next_activity_{}", next_page))
                .label(&list_activity_localised_text.next),
        )
    }

    trace!("{:?}", response);

    let mut message = component_interaction.message.clone();

    let a = message.edit(&ctx.http, response).await;
    trace!("{:?}", a);
    a.map_err(|e| CommandSendingError(format!("Error while sending the command {}", e)))
}
