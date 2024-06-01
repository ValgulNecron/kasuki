#[derive(sqlx::FromRow)]
pub struct GuildLanguage {
    pub guild: String,
    pub lang: String,
}