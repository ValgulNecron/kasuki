/// ActivationStatusModule is a struct that represents the activation status of various modules in the application.
/// It is derived from a row in a SQL database.
#[derive(sqlx::FromRow)]
pub struct ActivationStatusModule {
    /// id is an Option<String> that represents the ID of the module. It can be None if the ID is not set.
    pub id: Option<String>,
    /// ai_module is an Option<bool> that represents the activation status of the AI module. It can be None if the status is not set.
    pub ai_module: Option<bool>,
    /// anilist_module is an Option<bool> that represents the activation status of the Anilist module. It can be None if the status is not set.
    pub anilist_module: Option<bool>,
    /// game_module is an Option<bool> that represents the activation status of the game module. It can be None if the status is not set.
    pub game_module: Option<bool>,
    /// new_member is an Option<bool> that represents the activation status of the new member module. It can be None if the status is not set.
    pub new_member: Option<bool>,
    /// anime is an Option<bool> that represents the activation status of the anime module. It can be None if the status is not set.
    pub anime: Option<bool>,
}
