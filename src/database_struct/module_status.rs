#[derive(sqlx::FromRow)]
pub struct ActivationStatusModule {
    pub id: Option<String>,
    pub ai_module: Option<bool>,
    pub anilist_module: Option<bool>,
    pub game_module: Option<bool>,
    pub new_member: Option<bool>,
    pub anime: Option<bool>,
}
