use std::env;
use std::time::Duration;

use chrono::Utc;
use log::{debug, error};
use serenity::http::Http;
use serenity::model::channel::Embed;
use serenity::model::prelude::Webhook;
use serenity::utils;
use sqlx::FromRow;

use crate::constant::COLOR;
use crate::function::sqls::general::data::{
    get_data_activity, remove_data_module_activation_status, set_data_activity,
};
use crate::structure::anilist::struct_minimal_anime::MinimalAnimeWrapper;
use crate::structure::embed::anilist::struct_lang_send_activity::SendActivityLocalisedText;

#[derive(Debug, FromRow, Clone)]
pub struct ActivityData {
    anime_id: Option<String>,
    timestamp: Option<String>,
    server_id: Option<String>,
    webhook: Option<String>,
    episode: Option<String>,
    name: Option<String>,
    delays: Option<i32>,
}

pub async fn manage_activity() {
    loop {
        tokio::spawn(async move {
            send_activity().await;
        });
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

pub async fn send_activity() {
    let rows = get_data_activity().await;
    for row in rows {
        if Utc::now().timestamp().to_string() != row.timestamp.clone().unwrap() {
        } else {
            let row2 = row.clone();
            let guild_id = row.server_id.clone();
            if row.delays.unwrap() != 0 {
                tokio::spawn(async move {
                    tokio::time::sleep(Duration::from_secs((row2.delays.unwrap()) as u64)).await;
                    send_specific_activity(row, guild_id.unwrap(), row2).await
                });
            } else {
                send_specific_activity(row, guild_id.unwrap(), row2).await
            }
        }
    }
}

pub async fn update_info(row: ActivityData, guild_id: String) {
    tokio::time::sleep(Duration::from_secs(30 * 60)).await;
    let data = MinimalAnimeWrapper::new_minimal_anime_by_id_no_error(match &row.anime_id {
        Some(anime_id) => anime_id.parse().unwrap_or(0),
        None => 0,
    })
    .await;

    if data.get_episode().to_string() == row.episode.unwrap_or(String::from(0)) {
        let url = row.webhook.unwrap().parse().unwrap();
        let (id, token) = utils::parse_webhook(url).unwrap();
        let http = Http::new(&*token);

        http.delete_webhook(id).await?;

        remove_data_module_activation_status(row.server_id.unwrap(), row.anime_id.unwrap())
    } else {
        set_data_activity(
            data.get_id(),
            data.get_timestamp(),
            guild_id,
            row.webhook.unwrap(),
            data.get_episode(),
            data.get_name(),
            row.delays.unwrap() as i64,
        )
        .await
    }
}

pub async fn send_specific_activity(row: ActivityData, guild_id: String, row2: ActivityData) {
    let localised_text = SendActivityLocalisedText::get_send_activity_localised(guild_id.clone())
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
    let embed = Embed::fake(|e| {
        e.title(&localised_text.title)
            .url(format!(
                "https://anilist.co/anime/{}",
                row.anime_id.unwrap()
            ))
            .description(format!(
                "{}{} {}{} {}",
                &localised_text.ep,
                row.episode.unwrap(),
                &localised_text.of,
                row.name.unwrap(),
                localised_text.end,
            ))
            .color(COLOR)
    });
    webhook
        .execute(&http, false, |w| w.embeds(vec![embed]))
        .await
        .unwrap();
    tokio::spawn(async move { update_info(row2, guild_id).await });
}
