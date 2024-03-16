use serenity::all::{
    CommandInteraction, Context, CreateEmbed, CreateInteractionResponseFollowup,
    CreateInteractionResponseMessage, Timestamp,
};
use serenity::all::CreateInteractionResponse::Defer;

use crate::anilist_struct::run::minimal_anime::MinimalAnimeWrapper;
use crate::command_run::anilist::add_activity::get_name;
use crate::command_run::get_option::get_option_map_string_subcommand;
use crate::constant::COLOR;
use crate::database::dispatcher::data_dispatch::remove_data_activity_status;
use crate::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use crate::lang_struct::anilist::delete_activity::load_localization_delete_activity;

pub async fn run(ctx: &Context, command_interaction: &CommandInteraction) -> Result<(), AppError> {
    let map = get_option_map_string_subcommand(command_interaction);
    let anime = map
        .get(&String::from("anime_name"))
        .cloned()
        .unwrap_or(String::new());

    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };

    let delete_activity_localised_text = load_localization_delete_activity(guild_id).await?;
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };
    let builder_message = Defer(CreateInteractionResponseMessage::new());

    command_interaction
        .create_response(&ctx.http, builder_message)
        .await
        .map_err(|e| {
            AppError::new(
                format!("Error while sending the command {}", e),
                ErrorType::Command,
                ErrorResponseType::Message,
            )
        })?;
    let anime_id = if anime.parse::<i32>().is_ok() {
        anime.parse().unwrap()
    } else {
        MinimalAnimeWrapper::new_minimal_anime_by_search(anime.to_string())
            .await?
            .data
            .media
            .id
    };

    remove_activity(&guild_id, &anime_id).await?;

    let data = MinimalAnimeWrapper::new_minimal_anime_by_id(anime_id.to_string()).await?;
    let media = data.data.media;
    let title = media.title.unwrap();
    let anime_name = get_name(title);
    let builder_embed = CreateEmbed::new()
        .timestamp(Timestamp::now())
        .color(COLOR)
        .title(&delete_activity_localised_text.success)
        .url(format!("https://anilist.co/anime/{}", media.id))
        .description(
            delete_activity_localised_text
                .success_desc
                .replace("$anime$", anime_name.as_str()),
        );

    let builder_message = CreateInteractionResponseFollowup::new().embed(builder_embed);

    command_interaction
        .create_followup(&ctx.http, builder_message)
        .await
        .map_err(|e| {
            AppError::new(
                format!("Error while sending the command {}", e),
                ErrorType::Command,
                ErrorResponseType::Followup,
            )
        })?;
    Ok(())
}

pub async fn remove_activity(guild_id: &str, anime_id: &i32) -> Result<(), AppError> {
    remove_data_activity_status(guild_id.to_owned(), anime_id.to_string()).await
}
