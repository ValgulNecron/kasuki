use crate::constant::{
    ACTIVITY_LIST_LIMIT, COLOR, COMMAND_SENDING_ERROR, DIFFERED_COMMAND_SENDING_ERROR, OPTION_ERROR,
};
use crate::error_enum::AppError;
use crate::lang_struct::anilist::list_all_activity::load_localization_list_activity;
use crate::database::dispatcher::data_dispatch::get_all_server_activity;
use serenity::all::CreateInteractionResponse::Defer;
use serenity::all::{
    CommandInteraction, Context, CreateButton, CreateEmbed, CreateInteractionResponseFollowup,
    CreateInteractionResponseMessage, Timestamp,
};
use tracing::trace;

pub async fn run(ctx: &Context, command_interaction: &CommandInteraction) -> Result<(), AppError> {
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };

    let list_activity_localised_text = load_localization_list_activity(guild_id).await?;

    let guild_id = command_interaction.guild_id.ok_or(OPTION_ERROR.clone())?;

    let builder_message = Defer(CreateInteractionResponseMessage::new());

    command_interaction
        .create_response(&ctx.http, builder_message)
        .await
        .map_err(|_| COMMAND_SENDING_ERROR.clone())?;

    let list = get_all_server_activity(&guild_id.to_string()).await?;
    let len = list.len();
    let next_page = 1;

    let activity: Vec<String> = list
        .into_iter()
        .map(|(anime_id, _, _, _, _, name, _)| {
            format!(
                "[{}](https://anilist.co/anime/{})",
                name.unwrap_or_default(),
                anime_id.unwrap_or_default()
            )
        })
        .take(ACTIVITY_LIST_LIMIT as usize)
        .collect();

    let join_activity = activity.join("\n");

    let builder_message = CreateEmbed::new()
        .timestamp(Timestamp::now())
        .color(COLOR)
        .title(list_activity_localised_text.title)
        .description(join_activity);

    let mut response = CreateInteractionResponseFollowup::new().embed(builder_message);
    trace!("{:?}", len);
    trace!("{:?}", ACTIVITY_LIST_LIMIT);
    if len > ACTIVITY_LIST_LIMIT as usize {
        response = response.button(
            CreateButton::new(format!("next_activity_{}", next_page))
                .label(&list_activity_localised_text.next),
        )
    }

    let _ = command_interaction
        .create_followup(&ctx.http, response)
        .await
        .map_err(|_| DIFFERED_COMMAND_SENDING_ERROR.clone())?;
    Ok(())
}
