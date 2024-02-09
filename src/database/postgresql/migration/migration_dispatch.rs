use crate::database::postgresql::pool::get_postgresql_pool;
use crate::error_enum::AppError;
use crate::error_enum::AppError::NotACommandError;
use crate::error_enum::NotACommandError::{FailedToUpdateDatabase, SqlSelectError};

pub async fn migrate_postgres() -> Result<(), AppError> {
    // used to update the database when new row are added to a table.
    add_image_to_activity_data().await?;
    add_new_member_to_global_kill_switch().await?;
    add_new_column_to_module_activation().await?;
    Ok(())
}

pub async fn add_image_to_activity_data() -> Result<(), AppError> {
    let pool = get_postgresql_pool().await?;

    // Check if the "image" column exists in the "activity_data" table
    let row: (bool,) = sqlx::query_as(
        r#"
        SELECT EXISTS (
            SELECT 1 
            FROM information_schema.columns 
            WHERE table_name='activity_data' AND column_name='image'
        )
        "#,
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        NotACommandError(SqlSelectError(format!(
            "Failed to select from the table. {}",
            e
        )))
    })?;

    // If the "image" column doesn't exist, add it
    if !row.0 {
        sqlx::query("ALTER TABLE activity_data ADD COLUMN image TEXT")
            .execute(&pool)
            .await
            .map_err(|e| {
                NotACommandError(FailedToUpdateDatabase(format!(
                    "Failed to update the table. {}",
                    e
                )))
            })?;
    }

    pool.close().await;
    Ok(())
}

pub async fn add_new_member_to_global_kill_switch() -> Result<(), AppError> {
    let pool = get_postgresql_pool().await?;

    // Check if the "new_member" column exists in the "global_kill_switch" table
    let row: (bool,) = sqlx::query_as(
        r#"
        SELECT EXISTS (
            SELECT  1
            FROM information_schema.columns
            WHERE table_name='global_kill_switch' AND column_name='new_member'
        )
        "#,
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        NotACommandError(SqlSelectError(format!(
            "Failed to check existence of column. {}",
            e
        )))
    })?;

    // If the "new_member" column doesn't exist, add it
    if !row.0 {
        sqlx::query("ALTER TABLE global_kill_switch ADD COLUMN new_member TEXT")
            .execute(&pool)
            .await
            .map_err(|e| {
                NotACommandError(FailedToUpdateDatabase(format!(
                    "Failed to add column to the table. {}",
                    e
                )))
            })?;
    }

    pool.close().await;
    Ok(())
}

pub async fn add_new_column_to_module_activation() -> Result<(), AppError> {
    let pool = get_postgresql_pool().await?;

    // Check if the "new_column" column exists in the "module_activation" table
    let row: (bool,) = sqlx::query_as(
        r#"
        SELECT EXISTS (
            SELECT  1
            FROM information_schema.columns
            WHERE table_name='module_activation' AND column_name='new_column'
        )
        "#,
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        NotACommandError(SqlSelectError(format!(
            "Failed to check existence of column. {}",
            e
        )))
    })?;

    // If the "new_column" column doesn't exist, add it
    if !row.0 {
        sqlx::query("ALTER TABLE module_activation ADD COLUMN new_column TYPE")
            .execute(&pool)
            .await
            .map_err(|e| {
                NotACommandError(FailedToUpdateDatabase(format!(
                    "Failed to add column to the table. {}",
                    e
                )))
            })?;
    }

    pool.close().await;
    Ok(())
}
