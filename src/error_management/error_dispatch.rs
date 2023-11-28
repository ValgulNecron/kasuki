use crate::error_enum::AppError;
use log::error;

pub async fn command_dispatching(error: AppError) {
    match error {
        _ => {
            error!("{:?}", error)
        }
    }
}
