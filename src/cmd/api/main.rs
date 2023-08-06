use serenity::client::bridge::gateway::ShardManager;
use serenity::utils::shard_id;
use tokio::sync::Mutex;

use crate::cmd::general_module::pool::get_pool;

#[derive(serde::Serialize)]
struct Ping {
    id: u32,
    ping: String,
}


async fn handle(req: tide::Request<()>) -> tide::Result<String> {
    let shard_id = req.param("shardid").unwrap_or("0");
    let database_url = "./data.db";
    let pool = get_pool(database_url).await;
    let row: (Option<String>, Option<String>) = sqlx::query_as(
        "SELECT anilist_username, user_id FROM registered_user WHERE user_id = ?",
    )
        .bind(shard_id)
        .fetch_one(&pool)
        .await
        .unwrap_or((None, None));


    let ping = Ping {
        id: shard_id,
        ping: ping,
    }
    Ok("Hello world".to_string())
}

pub async fn create_server() -> tide::Result<()> {
    let mut app = tide::new();
    app.at("/api/ping/:shardid").get(handle);


    app.listen("0.0.0.0:5783").await?;
    Ok(())
}