use std::error::Error;
use std::sync::Arc;

use serenity::all::{Http, User, UserId};

use crate::helper::error_management::error_enum::UnknownResponseError;

pub async fn get_user_data(http: Arc<Http>, user: &UserId) -> Result<User, Box<dyn Error>> {
    let user = user
        .to_user(&http)
        .await
        .map_err(|e| UnknownResponseError::UserOrGuild(format!("{:#?}", e)))?;
    Ok(user)
}
