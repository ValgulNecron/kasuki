use crate::constant::SQLITE_DB_PATH;
use crate::database::manage::sqlite::pool::get_sqlite_pool;
use crate::helper::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};

/// Migrates the SQLite database.
///
/// This function is used to update the database when new rows are added to a table.
/// It performs the following operations in order:
/// 1. Calls the `add_image_to_activity_data` function to add an "image" column to the "activity_data" table if it does not exist.
/// 2. Calls the `add_new_member_to_module_activation` function to add a "new_member" column to the "module_activation" table if it does not exist.
/// 3. Calls the `add_new_member_to_global_kill_switch` function to add a "new_member" column to the "global_kill_switch" table if it does not exist.
/// 4. Calls the `add_anime_to_global_kill_switch` function to add an "anime" column to the "global_kill_switch" table if it does not exist.
/// 5. Calls the `add_anime_to_module_activation` function to add an "anime" column to the "module_activation" table if it does not exist.
///
/// # Returns
///
/// * A Result that is either an empty Ok variant if the operation was successful, or an Err variant with an AppError if the operation failed.
pub async fn migrate_sqlite() -> Result<(), AppError> {
    add_image_to_activity_data().await?;
    add_new_member_to_module_activation().await?;
    add_new_member_to_global_kill_switch().await?;
    add_anime_to_global_kill_switch().await?;
    add_anime_to_module_activation().await?;
    add_vn_to_global_kill_switch().await?;
    add_vn_to_module_activation().await?;
    Ok(())
}

/// Adds an "image" column to the "activity_data" table in the SQLite database if it does not exist.
///
/// This function performs the following operations in order:
/// 1. Retrieves a connection pool to the SQLite database using the `get_sqlite_pool` function.
/// 2. Checks if the "image" column exists in the "activity_data" table.
/// 3. If the "image" column does not exist, it adds it.
/// 4. Closes the connection pool.
///
/// # Returns
///
/// * A Result that is either an empty Ok variant if the operation was successful, or an Err variant with an AppError if the operation failed.
pub async fn add_image_to_activity_data() -> Result<(), AppError> {
    let pool = get_sqlite_pool(SQLITE_DB_PATH).await?;

    // Check if the "image" column exists in the "activity_data" table
    let row: u32 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM pragma_table_info('activity_data') WHERE name='image'",
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        AppError::new(
            format!("Failed to check existence of column. {}", e),
            ErrorType::Database,
            ErrorResponseType::None,
        )
    })?;

    // If the "image" column doesn't exist, add it
    if row == 0 {
        sqlx::query("ALTER TABLE activity_data ADD COLUMN image TEXT")
            .execute(&pool)
            .await
            .map_err(|e| {
                AppError::new(
                    format!("Failed to add column to the table. {}", e),
                    ErrorType::Database,
                    ErrorResponseType::None,
                )
            })?;
    }

    pool.close().await;
    Ok(())
}

/// Adds a "new_member" column to the "module_activation" table in the SQLite database if it does not exist.
///
/// This function performs the following operations in order:
/// 1. Retrieves a connection pool to the SQLite database using the `get_sqlite_pool` function.
/// 2. Checks if the "new_member" column exists in the "module_activation" table.
/// 3. If the "new_member" column does not exist, it adds it.
/// 4. Closes the connection pool.
///
/// # Returns
///
/// * A Result that is either an empty Ok variant if the operation was successful, or an Err variant with an AppError if the operation failed.
pub async fn add_new_member_to_module_activation() -> Result<(), AppError> {
    let pool = get_sqlite_pool(SQLITE_DB_PATH).await?;

    // Check if the "new_member" column exists in the "module_activation" table
    let row: u32 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM pragma_table_info('module_activation') WHERE name='new_member'",
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        AppError::new(
            format!("Failed to check existence of column. {}", e),
            ErrorType::Database,
            ErrorResponseType::None,
        )
    })?;

    // If the "new_member" column doesn't exist, add it
    if row == 0 {
        sqlx::query("ALTER TABLE module_activation ADD COLUMN new_member INTEGER")
            .execute(&pool)
            .await
            .map_err(|e| {
                AppError::new(
                    format!("Failed to add column to the table. {}", e),
                    ErrorType::Database,
                    ErrorResponseType::None,
                )
            })?;
    }

    pool.close().await;
    Ok(())
}

/// Adds a "new_member" column to the "global_kill_switch" table in the SQLite database if it does not exist.
///
/// This function performs the following operations in order:
/// 1. Retrieves a connection pool to the SQLite database using the `get_sqlite_pool` function.
/// 2. Checks if the "new_member" column exists in the "global_kill_switch" table.
/// 3. If the "new_member" column does not exist, it adds it.
/// 4. Closes the connection pool.
///
/// # Returns
///
/// * A Result that is either an empty Ok variant if the operation was successful, or an Err variant with an AppError if the operation failed.
pub async fn add_new_member_to_global_kill_switch() -> Result<(), AppError> {
    let pool = get_sqlite_pool(SQLITE_DB_PATH).await?;

    // Check if the "new_member" column exists in the "global_kill_switch" table
    let row: u32 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM pragma_table_info('global_kill_switch') WHERE name='new_member'",
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        AppError::new(
            format!("Failed to check existence of column. {}", e),
            ErrorType::Database,
            ErrorResponseType::None,
        )
    })?;

    // If the "new_member" column doesn't exist, add it
    if row == 0 {
        sqlx::query("ALTER TABLE global_kill_switch ADD COLUMN new_member INTEGER")
            .execute(&pool)
            .await
            .map_err(|e| {
                AppError::new(
                    format!("Failed to add column to the table. {}", e),
                    ErrorType::Database,
                    ErrorResponseType::None,
                )
            })?;
    }

    pool.close().await;
    Ok(())
}

/// Adds an "anime" column to the "module_activation" table in the SQLite database if it does not exist.
///
/// This function performs the following operations in order:
/// 1. Retrieves a connection pool to the SQLite database using the `get_sqlite_pool` function.
/// 2. Checks if the "anime" column exists in the "module_activation" table.
/// 3. If the "anime" column does not exist, it adds it.
/// 4. Closes the connection pool.
///
/// # Returns
///
/// * A Result that is either an empty Ok variant if the operation was successful, or an Err variant with an AppError if the operation failed.
pub async fn add_anime_to_module_activation() -> Result<(), AppError> {
    let pool = get_sqlite_pool(SQLITE_DB_PATH).await?;

    // Check if the "anime" column exists in the "module_activation" table
    let row: u32 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM pragma_table_info('module_activation') WHERE name='anime'",
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        AppError::new(
            format!("Failed to check existence of column. {}", e),
            ErrorType::Database,
            ErrorResponseType::None,
        )
    })?;

    // If the "anime" column doesn't exist, add it
    if row == 0 {
        sqlx::query("ALTER TABLE module_activation ADD COLUMN anime INTEGER")
            .execute(&pool)
            .await
            .map_err(|e| {
                AppError::new(
                    format!("Failed to add column to the table. {}", e),
                    ErrorType::Database,
                    ErrorResponseType::None,
                )
            })?;
    }

    pool.close().await;
    Ok(())
}

/// Adds an "anime" column to the "global_kill_switch" table in the SQLite database if it does not exist.
///
/// This function performs the following operations in order:
/// 1. Retrieves a connection pool to the SQLite database using the `get_sqlite_pool` function.
/// 2. Checks if the "anime" column exists in the "global_kill_switch" table.
/// 3. If the "anime" column does not exist, it adds it.
/// 4. Closes the connection pool.
///
/// # Returns
///
/// * A Result that is either an empty Ok variant if the operation was successful, or an Err variant with an AppError if the operation failed.
pub async fn add_anime_to_global_kill_switch() -> Result<(), AppError> {
    let pool = get_sqlite_pool(SQLITE_DB_PATH).await?;

    // Check if the "anime" column exists in the "global_kill_switch" table
    let row: u32 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM pragma_table_info('global_kill_switch') WHERE name='anime'",
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        AppError::new(
            format!("Failed to check existence of column. {}", e),
            ErrorType::Database,
            ErrorResponseType::None,
        )
    })?;

    // If the "anime" column doesn't exist, add it
    if row == 0 {
        sqlx::query("ALTER TABLE global_kill_switch ADD COLUMN anime INTEGER")
            .execute(&pool)
            .await
            .map_err(|e| {
                AppError::new(
                    format!("Failed to add column to the table. {}", e),
                    ErrorType::Database,
                    ErrorResponseType::None,
                )
            })?;
    }

    pool.close().await;
    Ok(())
}

pub async fn add_vn_to_global_kill_switch() -> Result<(), AppError> {
    let pool = get_sqlite_pool(SQLITE_DB_PATH).await?;

    // Check if the "vn" column exists in the "global_kill_switch" table
    let row: u32 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM pragma_table_info('global_kill_switch') WHERE name='vn'",
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        AppError::new(
            format!("Failed to check existence of column. {}", e),
            ErrorType::Database,
            ErrorResponseType::None,
        )
    })?;

    // If the "vn" column doesn't exist, add it
    if row == 0 {
        sqlx::query("ALTER TABLE global_kill_switch ADD COLUMN vn INTEGER")
            .execute(&pool)
            .await
            .map_err(|e| {
                AppError::new(
                    format!("Failed to add column to the table. {}", e),
                    ErrorType::Database,
                    ErrorResponseType::None,
                )
            })?;
    }

    pool.close().await;
    Ok(())
}

pub async fn add_vn_to_module_activation() -> Result<(), AppError> {
    let pool = get_sqlite_pool(SQLITE_DB_PATH).await?;

    // Check if the "vn" column exists in the "module_activation" table
    let row: u32 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM pragma_table_info('module_activation') WHERE name='vn'",
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        AppError::new(
            format!("Failed to check existence of column. {}", e),
            ErrorType::Database,
            ErrorResponseType::None,
        )
    })?;

    // If the "vn" column doesn't exist, add it
    if row == 0 {
        sqlx::query("ALTER TABLE module_activation ADD COLUMN vn INTEGER")
            .execute(&pool)
            .await
            .map_err(|e| {
                AppError::new(
                    format!("Failed to add column to the table. {}", e),
                    ErrorType::Database,
                    ErrorResponseType::None,
                )
            })?;
    }

    pool.close().await;
    Ok(())
}
