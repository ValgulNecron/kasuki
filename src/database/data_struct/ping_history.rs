#[derive(sqlx::FromRow)]
pub struct PingHistory {
    pub shard_id: String,
    pub ping: String,
    pub timestamp: String,
}
