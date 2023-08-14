use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::Read;
use std::thread::sleep;
use std::time::Duration;

use chrono::Utc;
use serenity::http::Http;
use serenity::model::channel::Embed;
use serenity::model::prelude::Webhook;
use sqlx::FromRow;

use crate::cmd::anilist_module::anime_activity::struct_minimal_anime::MinimalAnimeWrapper;
use crate::cmd::general_module::get_guild_langage::get_guild_langage;
use crate::cmd::general_module::lang_struct::SendActivityLocalisedText;
use crate::cmd::general_module::pool::get_pool;

#[derive(Debug, FromRow, Clone)]
pub struct ActivityData {
    anime_id: Option<String>,
    timestamp: Option<String>,
    server_id: Option<String>,
    webhook: Option<String>,
    episode: Option<String>,
    name: Option<String>,
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
        "SELECT anime_id, timestamp, server_id, webhook, episode, name FROM activity_data WHERE timestamp = ?",
    )
    .bind(now.clone())
    .fetch_all(&pool)
    .await
    .unwrap();
    for row in rows {
        let row2 = row.clone();
        let mut file =
            File::open("lang_file/anilist/send_activity.json").expect("Failed to open file");
        let mut json = String::new();
        file.read_to_string(&mut json).expect("Failed to read file");

        let json_data: HashMap<String, SendActivityLocalisedText> =
            serde_json::from_str(&json).expect("Failed to parse JSON");

        let guild_id = row.server_id.unwrap();
        let lang_choice = get_guild_langage(guild_id.clone()).await;

        if let Some(localised_text) = json_data.get(lang_choice.as_str()) {
            if row.timestamp.unwrap() != now {
                break;
            }
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
                    .url(format!("https://anilist.co/anime/{}", row.anime_id.unwrap()))
                    .description(format!(
                        "{}{} {}{} {}",
                        &localised_text.ep,
                        row.episode.unwrap(),
                        &localised_text.of,
                        row.name.unwrap(),
                        localised_text.end,
                    ))
            });
            webhook
                .execute(&http, false, |w| w.embeds(vec![embed]))
                .await
                .unwrap();
            tokio::spawn(async move {
                update_info(row2, guild_id)
            });
        }
    }
}

pub async fn update_info(row: ActivityData, guild_id: String){
    let database_url = "./data.db";
    let pool = get_pool(database_url).await;
    sleep(Duration::from_secs(30*60));
    let data = MinimalAnimeWrapper::new_minimal_anime_by_id_no_error(
                row.anime_id.clone().unwrap(),
            )
            .await;
    sqlx::query(
                "INSERT OR REPLACE INTO activity_data (anime_id, timestamp, server_id, webhook) VALUES (?, ?, ?, ?)",
            )
                .bind(row.anime_id)
                .bind(data.get_timestamp())
                .bind(guild_id.clone())
                .bind(row.webhook)
                .execute(&pool)
                .await
                .unwrap();
}
