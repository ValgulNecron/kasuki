use crate::database::postgresql::pool::get_postgresql_pool;
use crate::error_enum::AppError;
use crate::error_enum::AppError::Error;
use crate::error_enum::Error::{FailedToUpdateDatabase, SqlSelectError};

pub async fn migrate_postgres() -> Result<(), AppError> {
    // used to update the database when new row are added to a table.
    add_image_to_activity_data().await?;
    Ok(())
}

pub async fn add_image_to_activity_data() -> Result<(), AppError> {
    let pool = get_postgresql_pool().await?;

    // Check if the "image" column exists in the "activity_data" table
    let row: (bool, ) = sqlx::query_as(
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
        .map_err(|e| Error(SqlSelectError(format!("Failed to select from the table. {}", e))))?;

    // If the "image" column doesn't exist, add it
    if !row.0 {
        sqlx::query("ALTER TABLE activity_data ADD COLUMN image TEXT")
            .execute(&pool)
            .await
            .map_err(|e| Error(FailedToUpdateDatabase(format!("Failed to update the table. {}", e))))?;
    }

    pool.close().await;
    Ok(())
}
