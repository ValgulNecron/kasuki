use crate::constant::DATA_SQLITE_DB;
use crate::database::sqlite::pool::get_sqlite_pool;
use crate::error_enum::AppError;
use crate::error_enum::AppError::{FailedToUpdateDatabase, SqlSelectError};

pub async fn migrate_sqlite() -> Result<(), AppError> {
    // used to update the database when new row are added to a table.
    add_image_to_activity_data().await?;
    Ok(())
}

pub async fn add_image_to_activity_data() -> Result<(), AppError> {
    let pool = get_sqlite_pool(DATA_SQLITE_DB).await?;

    // Check if the "image" column exists in the "activity_data" table
    let row: (i64, String, String, i64, i64, i64) = sqlx::query_as(
        r#"
        SELECT COUNT(*) FROM pragma_table_info('activity_data')
        WHERE name='image'
        "#,
    )
    .fetch_one(&pool)
    .await
    .map_err(|_| SqlSelectError(String::from("Failed to select from the table.")))?;

    // If the "image" column doesn't exist, add it
    if row.0 == 0 {
        sqlx::query("ALTER TABLE activity_data ADD COLUMN image TEXT")
            .execute(&pool)
            .await
            .map_err(|_| FailedToUpdateDatabase(String::from("Failed to update the table.")))?;
    }

    pool.close().await;
    Ok(())
}
