use std::error::Error;
use std::sync::Arc;

use serenity::all::{Http, User, UserId};

pub async fn get_user_data(http: Arc<Http>, user: &UserId) -> Result<User, Box<dyn Error>> {
    let user = user.to_user(&http).await?;
    Ok(user)
}
