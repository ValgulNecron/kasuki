use tracing::error;

use crate::error_enum::{AppError, DifferedError, Error};

pub async fn command_dispatching(error: AppError) {
    error!("{:?}", error);
    match error {
        AppError::Error(e) => send_error(e).await,
        AppError::DifferedError(e) => send_differed_error(e).await,
        AppError::ComponentError(_) => {}
        AppError::NotACommandError(_) => {}
        AppError::JoiningError(_) => {}
    }
}

async fn send_error(e: Error) {}

async fn send_differed_error(e: DifferedError) {}
