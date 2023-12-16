use tracing::error;

use crate::error_enum::AppError;

pub async fn command_dispatching(error: AppError) {
    match error {
        _ => {
            error!("{:?}", error)
        }
    }
}
