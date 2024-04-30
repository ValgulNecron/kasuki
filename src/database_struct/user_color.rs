/// UserColor is a struct that represents a user's color preferences in the application.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct UserColor {
    /// user_id is an Option<String> that represents the user's ID. It can be None if the user's ID is not set.
    pub user_id: Option<String>,
    /// color is an Option<String> that represents the user's preferred color. It can be None if the user's preferred color is not set.
    pub color: Option<String>,
    /// pfp_url is an Option<String> that represents the URL of the user's profile picture. It can be None if the URL is not set.
    pub pfp_url: Option<String>,
    /// image is an Option<String> that represents the user's image. It can be None if the image is not set.
    pub image: Option<String>,
}