use crate::config::BotConfigDetails;
use crate::helper::error_management::error_enum::UnknownResponseError;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::{ConnectOptions, Pool, Postgres};
use std::error::Error;
use std::fs::OpenOptions;
use std::{fs, process};
use std::io::{Cursor, Read, Write};
use tracing::{error, trace};

/// Retrieves a connection pool to the PostgreSQL database.
///
/// This function performs the following operations in order:
/// 1. Retrieves the `DATABASE_URL` environment variable.
/// 2. Creates a new `PgPoolOptions` instance and sets the maximum number of connections to 20.
/// 3. Connects to the PostgreSQL database using the `DATABASE_URL` and the `PgPoolOptions` instance.
///
/// # Returns
///
/// * A Result that is either a `Pool<Postgres>` if the operation was successful, or an Err variant with an `AppError` if the operation failed. The `AppError` contains a description of the error, the type of the error (`ErrorType::Database`), and the response type of the error (`ErrorResponseType::Unknown`).
pub async fn get_postgresql_pool(
    db_config: BotConfigDetails,
) -> Result<Pool<Postgres>, Box<dyn Error>> {
    let host = match db_config.host.clone() {
        Some(host) => host,
        None => {
            error!("No host provided");
            process::exit(7)
        }
    };
    let port = match db_config.port.clone() {
        Some(port) => port,
        None => {
            error!("No port provided");
            process::exit(7)
        }
    };
    let user = match db_config.user.clone() {
        Some(user) => user,
        None => {
            error!("No user provided");
            process::exit(7)
        }
    };
    let password = match db_config.password.clone() {
        Some(password) => password,
        None => {
            error!("No password provided");
            process::exit(7)
        }
    };
    let database = "kasuki";
    let pg_con_op = PgConnectOptions::new()
        .host(host.as_str())
        .password(password.as_str())
        .username(user.as_str())
        .port(port)
        .database(&database)
        .to_url_lossy();
        let pool = PgPoolOptions::new()
        .max_connections(20)
        .connect_lazy(pg_con_op.as_str())
        .map_err(|e| {
            UnknownResponseError::Database(format!(
                "Failed to connect to the postgres pool: {:#?}",
                e
            ))
        })?;

    Ok(pool)
}

pub async fn get_postgresql_pool_without_db(
    db_config: BotConfigDetails,
) -> Result<Pool<Postgres>, Box<dyn Error>> {
    let host = match db_config.host.clone() {
        Some(host) => host,
        None => {
            error!("No host provided");
            process::exit(7)
        }
    };
    let port = match db_config.port.clone() {
        Some(port) => port,
        None => {
            error!("No port provided");
            process::exit(7)
        }
    };
    let user = match db_config.user.clone() {
        Some(user) => user,
        None => {
            error!("No user provided");
            process::exit(7)
        }
    };
    let password = match db_config.password.clone() {
        Some(password) => password,
        None => {
            error!("No password provided");
            process::exit(7)
        }
    };
    let pg_con_op = PgConnectOptions::new()
        .host(host.as_str())
        .password(password.as_str())
        .username(user.as_str())
        .port(port)
        .to_url_lossy();
    trace!(?pg_con_op);
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .connect_lazy(pg_con_op.as_str())
        .map_err(|e| {
            UnknownResponseError::Database(format!(
                "Failed to connect to the postgres pool: {:#?}",
                e
            ))
        })?;

    Ok(pool)
}

