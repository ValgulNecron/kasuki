use std::sync::Arc;

use serenity::all::{Http, User, UserId};

use crate::helper::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};

pub async fn get_user_data(http: Arc<Http>, user: &UserId) -> Result<User, AppError> {
    let user = user.to_user(&http).await.map_err(|e| {
        AppError::new(
            format!("Could not get the user. {}", e),
            ErrorType::Option,
            ErrorResponseType::Message,
        )
    })?;
    Ok(user)
}
