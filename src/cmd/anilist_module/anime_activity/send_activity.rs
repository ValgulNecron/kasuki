use std::env;
use std::thread::sleep;
use std::time::Duration;

use chrono::Utc;
use serenity::http::Http;
use serenity::model::channel::Embed;
use serenity::model::prelude::Webhook;
use serenity::utils::Colour;
use sqlx::FromRow;

use crate::cmd::anilist_module::anime_activity::struct_minimal_anime::MinimalAnimeWrapper;
use crate::cmd::general_module::function::sql::get_pool;
use crate::cmd::lang_struct::embed::anilist::anilist_activity::struct_lang_send_activity::SendActivityLocalisedText;

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
    let database_url = "./data.db";
    let pool = get_pool(database_url).await;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS activity_data (
        anime_id TEXT,
        timestamp TEXT,
        server_id TEXT,
        webhook TEXT,
        episode TEXT,
        name TEXT,
        delays INTEGER DEFAULT 0,
        PRIMARY KEY (anime_id, server_id)
    )",
    )
    .execute(&pool)
    .await
    .unwrap();
    loop {
        tokio::spawn(async move {
            send_activity().await;
        });
        sleep(Duration::from_secs(1));
    }
}

pub async fn send_activity() {
    let database_url = "./data.db";
    let pool = get_pool(database_url).await;
    let now = Utc::now().timestamp().to_string();
    let rows: Vec<ActivityData> = sqlx::query_as(
        "SELECT anime_id, timestamp, server_id, webhook, episode, name, delays FROM activity_data WHERE timestamp = ?",
    )
        .bind(now.clone())
        .fetch_all(&pool)
        .await
        .unwrap();
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
    let database_url = "./data.db";
    let pool = get_pool(database_url).await;
    sleep(Duration::from_secs(30 * 60));
    let data =
        MinimalAnimeWrapper::new_minimal_anime_by_id_no_error(row.anime_id.clone().unwrap()).await;
    sqlx::query(
        "INSERT OR REPLACE INTO activity_data (anime_id, timestamp, server_id, webhook, episode, name, delays) VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
        .bind(row.anime_id.unwrap())
        .bind(data.get_timestamp())
        .bind(guild_id)
        .bind(row.webhook.unwrap())
        .bind(data.get_episode())
        .bind(data.get_name())
        .bind(data.get_name())
        .bind(row.delays.unwrap())
        .execute(&pool)
        .await
        .unwrap();
}

pub async fn send_specific_activity(row: ActivityData, guild_id: String, row2: ActivityData) {
    let color = Colour::FABLED_PINK;
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
            .color(color)
    });
    webhook
        .execute(&http, false, |w| w.embeds(vec![embed]))
        .await
        .unwrap();
    tokio::spawn(async move { update_info(row2, guild_id).await });
}
