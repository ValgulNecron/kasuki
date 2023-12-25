use std::env;
use std::time::Duration;

use chrono::Utc;
use serenity::all::{CreateEmbed, ExecuteWebhook, Http, Webhook};
use tokio::time::sleep;
use tracing::trace;

use crate::anilist_struct::run::minimal_anime::{ActivityData, MinimalAnimeWrapper};
use crate::constant::{COLOR, OPTION_ERROR};
use crate::error_enum::AppError;
use crate::lang_struct::anilist::send_activity::load_localization_send_activity;
use crate::sqls::general::data::{
    get_data_activity, remove_data_activity_status, set_data_activity,
};

pub async fn manage_activity() {
    trace!("Started the activity management.");
    loop {
        tokio::spawn(async move { send_activity().await });
        sleep(Duration::from_secs(1)).await;
    }
}

pub async fn send_activity() {
    let now = Utc::now().timestamp().to_string();
    trace!("{:#?}", now);
    let rows = get_data_activity(now.clone()).await.unwrap();
    for row in rows {
        if Utc::now().timestamp().to_string() != row.timestamp.clone().unwrap() {
        } else {
            let row2 = row.clone();
            let guild_id = row.server_id.clone();
            if row.delays.unwrap() != 0 {
                tokio::spawn(async move {
                    tokio::time::sleep(Duration::from_secs((row2.delays.unwrap()) as u64)).await;
                    send_specific_activity(row, guild_id.unwrap(), row2)
                        .await
                        .unwrap()
                });
            } else {
                send_specific_activity(row, guild_id.unwrap(), row2)
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
) -> Result<(), AppError> {
    let localised_text = load_localization_send_activity(guild_id.clone())
        .await
        .unwrap();
    let my_path = "./.env";
    let path = std::path::Path::new(my_path);
    let _ = dotenv::from_path(path);
    let token = env::var("DISCORD_TOKEN").expect("discord token");
    let http = Http::new(token.as_str());
    let webhook = Webhook::from_url(&http, row.webhook.clone().unwrap().as_ref())
        .await
        .unwrap();
    let embed = CreateEmbed::new()
        .color(COLOR)
        .description(
            &localised_text
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
        .execute(&http, false, builder_message)
        .await
        .unwrap();
    tokio::spawn(async move { update_info(row2, guild_id).await });
    Ok(())
}

pub async fn update_info(row: ActivityData, guild_id: String) -> Result<(), AppError> {
    let data = MinimalAnimeWrapper::new_minimal_anime_by_id(
        row.anime_id.clone().ok_or(OPTION_ERROR.clone())?,
    )
    .await?;
    let media = data.data.media;
    let next_airing = match media.next_airing_episode {
        Some(na) => na,
        None => return remove_activity(row, guild_id).await,
    };
    let title = media.title.ok_or(OPTION_ERROR.clone())?;
    let rj = title.romaji;
    let en = title.english;
    let name = en.unwrap_or(rj.unwrap_or(String::from("nothing")));
    set_data_activity(
        media.id,
        next_airing.airing_at.unwrap(),
        guild_id,
        row.webhook.unwrap(),
        next_airing.episode.unwrap(),
        name.clone(),
        row.delays.unwrap_or(0) as i64,
    )
    .await
}

pub async fn remove_activity(row: ActivityData, guild_id: String) -> Result<(), AppError> {
    trace!("removing {:#?} for {:#?}", row, guild_id);
    remove_data_activity_status(guild_id, row.anime_id.unwrap_or(1.to_string())).await
}
