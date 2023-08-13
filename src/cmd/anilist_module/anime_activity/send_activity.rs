use std::env;
use std::ops::Sub;
use std::thread::sleep;
use std::time::Duration;
use chrono::{TimeZone, Utc};
use serenity::http::Http;
use serenity::model::channel::Embed;
use serenity::model::prelude::Webhook;
use sqlx::FromRow;
use crate::cmd::anilist_module::anime_activity::struct_minimal_anime::MinimalAnimeWrapper;
use crate::cmd::general_module::pool::get_pool;

#[derive(Debug, FromRow)]
struct ActivityData {
    anime_id: Option<String>,
    timestamp: Option<String>,
    server_id: Option<String>,
    webhook: Option<String>,
}

pub async fn send_activity() {
    let database_url = "./data.db";
    let pool =get_pool(database_url).await;
    loop {
        let rows: Vec<ActivityData> = sqlx::query_as(
            "SELECT anime_id, timestamp, server_id, webhook FROM activity_data"
        ).fetch_all(&pool).await.unwrap();
        for row in rows {
            let timestamp = row.timestamp.unwrap().clone();
            let now = Utc::now();
            if timestamp == now.timestamp().to_string() {
                let my_path = "./.env";
                let path = std::path::Path::new(my_path);
                let _ = dotenv::from_path(path);
                let token = env::var("DISCORD_TOKEN").expect("discord token");
                let data = MinimalAnimeWrapper::new_minimal_anime_by_id_no_error(row.anime_id.clone().unwrap()).await;
                let http = Http::new(token.as_str());
                let webhook = Webhook::from_url(&http, row.webhook.clone().unwrap().as_ref()).await.unwrap();
                let embed = Embed::fake(|e| {
                e.title("New episode")
                    .url(format!("https://anilist.co/anime/{}", data.get_id()))
                    .description(format!("Episode {} of {} just released", data.get_episode(), data.get_name()))
                });
                webhook.execute(&http, false, |w| {
                    w.embeds(vec![embed])
                }).await.unwrap();
                sqlx::query(
                "INSERT OR REPLACE INTO activity_data (anime_id, timestamp, server_id, webhook) VALUES (?, ?, ?, ?)",
                )
                .bind(row.anime_id)
                .bind(data.get_timestamp())
                .bind(row.server_id)
                .bind(row.webhook)
                .execute(&pool)
                .await
                .unwrap();
            }
        }
        sleep(Duration::from_secs(1));
    }
}