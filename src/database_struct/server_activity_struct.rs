pub struct ServerActivity {
    pub anime_id: Option<String>,
    pub timestamp: Option<String>,
    pub server_id: Option<String>,
    pub webhook: Option<String>,
    pub episode: Option<String>,
    pub name: Option<String>,
    pub delays: Option<u32>,
}

pub struct ServerActivityFull {
    pub anime_id: i32,
    pub timestamp: i64,
    pub guild_id: String,
    pub webhook: String,
    pub episode: i32,
    pub name: String,
    pub delays: i64,
    pub image: String,
}
