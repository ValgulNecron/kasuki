use base64::engine::general_purpose::STANDARD;
use base64::read::DecoderReader;
use std::io::{Cursor, Read};
use std::time::Duration;

use chrono::Utc;
use serenity::all::{Context, CreateAttachment, CreateEmbed, EditWebhook, ExecuteWebhook, Webhook};
use tokio::time::sleep;
use tracing::trace;

use crate::anilist_struct::run::minimal_anime::{ActivityData, MinimalAnimeWrapper};
use crate::constant::COLOR;
use crate::database::dispatcher::data_dispatch::{
    get_data_activity, remove_data_activity_status, set_data_activity,
};
use crate::database_struct::server_activity_struct::ServerActivityFull;
use crate::error_enum::AppError;
use crate::error_enum::AppError::OptionError;
use crate::lang_struct::anilist::send_activity::load_localization_send_activity;

pub async fn manage_activity(ctx: Context) {
    loop {
        let ctx = ctx.clone();
        tokio::spawn(async move { send_activity(&ctx).await });
        sleep(Duration::from_secs(1)).await;
    }
}

pub async fn send_activity(ctx: &Context) {
    let now = Utc::now().timestamp().to_string();
    let rows = get_data_activity(now.clone()).await.unwrap();
    for row in rows {
        if Utc::now().timestamp().to_string() != row.timestamp.clone().unwrap() {
        } else {
            let row2 = row.clone();
            let guild_id = row.server_id.clone();
            if row.delays.unwrap() != 0 {
                let ctx = ctx.clone();
                tokio::spawn(async move {
                    tokio::time::sleep(Duration::from_secs((row2.delays.unwrap()) as u64)).await;
                    send_specific_activity(row, guild_id.unwrap(), row2, &ctx)
                        .await
                        .unwrap()
                });
            } else {
                send_specific_activity(row, guild_id.unwrap(), row2, ctx)
                    .await
                    .unwrap()
            }
        }
    }
}

pub async fn send_specific_activity(
    row: ActivityData,
    guild_id: String,
    row2: ActivityData,
    ctx: &Context,
) -> Result<(), AppError> {
    let localised_text = load_localization_send_activity(guild_id.clone())
        .await
        .unwrap();
    let webhook_url = row.webhook.clone().unwrap();
    let mut webhook = Webhook::from_url(&ctx.http, webhook_url.as_str())
        .await
        .unwrap();

    let image = row.image.unwrap_or_default();
    trace!(image);

    let cursor = Cursor::new(image);
    let mut decoder = DecoderReader::new(cursor, &STANDARD);

    // Read the decoded bytes into a Vec
    let mut decoded_bytes = Vec::new();
    decoder
        .read_to_end(&mut decoded_bytes)
        .expect("Failed to decode base64");
    let attachement = CreateAttachment::bytes(decoded_bytes, "avatar");
    let edit_webhook = EditWebhook::new()
        .name(row.name.clone().unwrap())
        .avatar(&attachement);
    webhook.edit(&ctx.http, edit_webhook).await.unwrap();

    let embed = CreateEmbed::new()
        .color(COLOR)
        .description(
            localised_text
                .desc
                .replace("$ep$", row.episode.unwrap_or(String::from("0")).as_str())
                .replace("$anime$", row.name.unwrap_or(String::from("none")).as_str()),
        )
        .url(format!(
            "https://anilist.co/anime/{}",
            row.anime_id.unwrap_or(String::from("0"))
        ))
        .title(&localised_text.title);

    let builder_message = ExecuteWebhook::new().embed(embed);

    webhook
        .execute(&ctx.http, false, builder_message)
        .await
        .unwrap();

    tokio::spawn(async move { update_info(row2, guild_id).await });
    Ok(())
}

pub async fn update_info(row: ActivityData, guild_id: String) -> Result<(), AppError> {
    let data = MinimalAnimeWrapper::new_minimal_anime_by_id(
        row.anime_id.clone().ok_or(OptionError(String::from("There is no option")))?
    )
    .await?;
    let media = data.data.media;
    let next_airing = match media.next_airing_episode {
        Some(na) => na,
        None => return remove_activity(row, guild_id).await,
    };
    let title = media.title.ok_or(OptionError(String::from("There is no option")))?;
    let rj = title.romaji;
    let en = title.english;
    let name = en.unwrap_or(rj.unwrap_or(String::from("nothing")));
    set_data_activity(ServerActivityFull {
        anime_id: media.id,
        timestamp: next_airing.airing_at.unwrap(),
        guild_id,
        webhook: row.webhook.unwrap(),
        episode: next_airing.episode.unwrap(),
        name,
        delays: row.delays.unwrap_or(0) as i64,
        image: row.image.unwrap_or_default(),
    })
    .await
}

pub async fn remove_activity(row: ActivityData, guild_id: String) -> Result<(), AppError> {
    trace!("removing {:#?} for {}", row, guild_id);
    remove_data_activity_status(guild_id, row.anime_id.unwrap_or(1.to_string())).await
}
