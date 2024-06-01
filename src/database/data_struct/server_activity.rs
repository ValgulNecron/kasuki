/// ServerActivity is a struct that represents the activity of a server.
/// It is derived from a row in a SQL database.
#[derive(sqlx::FromRow)]
pub struct ServerActivity {
    /// anime_id is an Option<String> that represents the ID of the anime. It can be None if the anime ID is not set.
    pub anime_id: Option<String>,
    /// timestamp is an Option<String> that represents the timestamp of the activity. It can be None if the timestamp is not set.
    pub timestamp: Option<String>,
    /// server_id is an Option<String> that represents the ID of the server. It can be None if the server ID is not set.
    pub server_id: Option<String>,
    /// webhook is an Option<String> that represents the webhook of the activity. It can be None if the webhook is not set.
    pub webhook: Option<String>,
    /// episode is an Option<String> that represents the episode of the anime. It can be None if the episode is not set.
    pub episode: Option<String>,
    /// name is an Option<String> that represents the name of the activity. It can be None if the name is not set.
    pub name: Option<String>,
    /// delays is an Option<i64> that represents the delay of the activity. It can be None if the delay is not set.
    pub delays: Option<i64>,
}

// ServerActivityFull is a struct that represents the full activity of a server.
// It is derived from a row in a SQL database.
#[derive(sqlx::FromRow)]
pub struct ServerActivityFull {
    /// anime_id is an i32 that represents the ID of the anime.
    pub anime_id: i32,
    /// timestamp is an i64 that represents the timestamp of the activity.
    pub timestamp: i64,
    /// guild_id is a String that represents the ID of the guild.
    pub guild_id: String,
    /// webhook is a String that represents the webhook of the activity.
    pub webhook: String,
    /// episode is an i32 that represents the episode of the anime.
    pub episode: i32,
    /// name is a String that represents the name of the activity.
    pub name: String,
    /// delays is an i64 that represents the delay of the activity.
    pub delays: i64,
    /// image is a String that represents the image of the activity.
    pub image: String,
}

#[derive(sqlx::FromRow)]
pub struct SmallServerActivity {
    pub anime_id: i32,
    pub timestamp: i64,
    pub guild_id: String,
}
