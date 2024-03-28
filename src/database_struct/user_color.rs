#[derive(Debug, Clone, sqlx::FromRow)]
pub struct UserColor {
    pub user_id: Option<String>,
    pub color: Option<String>,
    pub pfp_url: Option<String>,
    pub image: Option<String>,
}
