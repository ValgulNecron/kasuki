use tracing::error;

use crate::error_enum::AppError;
use crate::error_enum::AppError::{DifferedError, Error};

pub async fn command_dispatching(error: AppError) {
    error!("{:?}", error);
    match error {
        Error(e) => send_error(e).await,
        DifferedError(e) => send_differed_error(e).await,
    }
}

async fn send_error(e: Error(String)) {}

async fn send_differed_error(e: DifferedError(String)) {}
