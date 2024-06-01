#[derive(sqlx::FromRow)]
pub struct RegisteredUser {
    pub user_id: String,
    pub anilist_id: String,
}