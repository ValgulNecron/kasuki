use std::env;
use chrono::Utc;
use serenity::http::Http;
use serenity::model::channel::Embed;
use serenity::model::prelude::Webhook;
use serenity::model::webhook;
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
            if Utc::now().timestamp().to_string() != row.timestamp.unwrap() {

            } else {
                let my_path = "./.env";
                let path = std::path::Path::new(my_path);
                let _ = dotenv::from_path(path);
                let token = env::var("DISCORD_TOKEN").expect("discord token");
                let data = MinimalAnimeWrapper::new_minimal_anime_by_id_no_error(row.anime_id.unwrap()).await;
                let http = Http::new(token.as_str());
                let webhook = Webhook::from_url(&http, row.webhook.unwrap().as_ref()).await.unwrap();
                let embed = Embed::fake(|e| {
                e.title("New episode")
                    .url(format!("https://anilist.co/anime/{}", data.get_id()))
                    .description(format!("Episode {} of {} just released", data.get_episode(), data.get_name()))
                });
                webhook.execute(&http, false, |w| {
                    w.embeds(vec![embed])
                }).await.unwrap();
            }
        }
    }
}