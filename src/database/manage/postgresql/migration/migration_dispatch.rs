use std::error::Error;

use crate::database::manage::postgresql::pool::get_postgresql_pool;
use crate::helper::error_management::error_enum;

/// Migrates the PostgreSQL database.
///
/// This function does not take any parameters.
/// It sequentially calls other functions to add new columns to various tables in the database.
/// These functions are used to update the database when new rows are added to a table.
///
/// The functions called are:
/// * `add_image_to_activity_data` - Adds an "image" column to the "activity_data" table.
/// * `add_new_member_to_global_kill_switch` - Adds a "new_member" column to the "global_kill_switch" table.
/// * `add_new_member_to_module_activation` - Adds a "new_member" column to the "module_activation" table.
/// * `add_anime_to_global_kill_switch` - Adds an "anime" column to the "global_kill_switch" table.
/// * `add_anime_to_module_activation` - Adds an "anime" column to the "module_activation" table.
///
/// # Returns
///
/// * A Result that is either an empty Ok variant if the operation was successful, or an Err variant with an AppError.
pub async fn migrate_postgres() -> Result<(), Box<dyn Error>> {
    // used to update the database when new row are added to a table.
    add_image_to_activity_data().await?;
    add_new_member_to_global_kill_switch().await?;
    add_new_member_to_module_activation().await?;
    add_anime_to_global_kill_switch().await?;
    add_anime_to_module_activation().await?;
    add_vn_to_global_kill_switch().await?;
    add_vn_to_module_activation().await?;
    update_name_of_id_in_global_kill_switch().await?;
    update_name_of_id_in_global_module_activation().await?;
    Ok(())
}

async fn update_name_of_id_in_global_kill_switch() -> Result<(), Box<dyn Error>> {
    // change the name of the row id to guild_id
    let pool = get_postgresql_pool().await?;
    sqlx::query("ALTER TABLE global_kill_switch RENAME COLUMN id TO guild_id")
        .execute(&pool)
        .await
        .map_err(|e| error_enum::Error::Database(format!("Failed to execute query. {:#?}", e)))?;
    Ok(())
}
async fn update_name_of_id_in_global_module_activation() -> Result<(), Box<dyn Error>> {
    // change the name of the row id to guild_id
    let pool = get_postgresql_pool().await?;
    sqlx::query("ALTER TABLE module_activation RENAME COLUMN id TO guild_id")
        .execute(&pool)
        .await
        .map_err(|e| error_enum::Error::Database(format!("Failed to execute query. {:#?}", e)))?;
    Ok(())
}
/// Adds an "image" column to the "activity_data" table in the PostgreSQL database.
///
/// This function does not take any parameters.
/// It first checks if the "image" column already exists in the "activity_data" table.
/// If the "image" column does not exist, it adds the column.
///
/// # Returns
///
/// * A Result that is either an empty Ok variant if the operation was successful, or an Err variant with an AppError.
pub async fn add_image_to_activity_data() -> Result<(), Box<dyn Error>> {
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
    .map_err(|e| error_enum::Error::Database(format!("Failed to execute query. {:#?}", e)))?;

    // If the "image" column doesn't exist, add it
    if !row.0 {
        sqlx::query("ALTER TABLE activity_data ADD COLUMN image TEXT")
            .execute(&pool)
            .await
            .map_err(|e| {
                error_enum::Error::Database(format!("Failed to execute query. {:#?}", e))
            })?;
    }

    pool.close().await;
    Ok(())
}

/// Adds a "new_member" column to the "global_kill_switch" table in the PostgreSQL database.
///
/// This function does not take any parameters.
/// It first checks if the "new_member" column already exists in the "global_kill_switch" table.
/// If the "new_member" column does not exist, it adds the column.
///
/// # Returns
///
/// * A Result that is either an empty Ok variant if the operation was successful, or an Err variant with an AppError.
pub async fn add_new_member_to_global_kill_switch() -> Result<(), Box<dyn Error>> {
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
    .map_err(|e| error_enum::Error::Database(format!("Failed to execute query. {:#?}", e)))?;

    // If the "new_member" column doesn't exist, add it
    if !row.0 {
        sqlx::query("ALTER TABLE global_kill_switch ADD COLUMN new_member BIGINT")
            .execute(&pool)
            .await
            .map_err(|e| {
                error_enum::Error::Database(format!("Failed to execute query. {:#?}", e))
            })?;
    }

    pool.close().await;
    Ok(())
}

/// Adds a "new_member" column to the "module_activation" table in the PostgreSQL database.
///
/// This function does not take any parameters.
/// It first checks if the "new_member" column already exists in the "module_activation" table.
/// If the "new_member" column does not exist, it adds the column.
///
/// # Returns
///
/// * A Result that is either an empty Ok variant if the operation was successful, or an Err variant with an AppError.
pub async fn add_new_member_to_module_activation() -> Result<(), Box<dyn Error>> {
    let pool = get_postgresql_pool().await?;

    // Check if the "new_member" column exists in the "module_activation" table
    let row: (bool,) = sqlx::query_as(
        r#"
        SELECT EXISTS (
            SELECT  1
            FROM information_schema.columns
            WHERE table_name='module_activation' AND column_name='new_member'
        )
        "#,
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| error_enum::Error::Database(format!("Failed to execute query. {:#?}", e)))?;

    // If the "new_member" column doesn't exist, add it
    if !row.0 {
        sqlx::query("ALTER TABLE module_activation ADD COLUMN new_member BIGINT")
            .execute(&pool)
            .await
            .map_err(|e| {
                error_enum::Error::Database(format!("Failed to execute query. {:#?}", e))
            })?;
    }

    pool.close().await;
    Ok(())
}

/// Adds an "anime" column to the "module_activation" table in the PostgreSQL database.
///
/// This function does not take any parameters.
/// It first checks if the "anime" column already exists in the "module_activation" table.
/// If the "anime" column does not exist, it adds the column.
///
/// # Returns
///
/// * A Result that is either an empty Ok variant if the operation was successful, or an Err variant with an AppError.
pub async fn add_anime_to_module_activation() -> Result<(), Box<dyn Error>> {
    let pool = get_postgresql_pool().await?;

    // Check if the "anime" column exists in the "module_activation" table
    let row: (bool,) = sqlx::query_as(
        r#"
        SELECT EXISTS (
            SELECT  1
            FROM information_schema.columns
            WHERE table_name='module_activation' AND column_name='anime'
        )
        "#,
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| error_enum::Error::Database(format!("Failed to execute query. {:#?}", e)))?;

    // If the "anime" column doesn't exist, add it
    if !row.0 {
        sqlx::query("ALTER TABLE module_activation ADD COLUMN anime BIGINT")
            .execute(&pool)
            .await
            .map_err(|e| {
                error_enum::Error::Database(format!("Failed to execute query. {:#?}", e))
            })?;
    }

    pool.close().await;
    Ok(())
}

/// Adds an "anime" column to the "global_kill_switch" table in the PostgreSQL database.
///
/// This function does not take any parameters.
/// It first checks if the "anime" column already exists in the "global_kill_switch" table.
/// If the "anime" column does not exist, it adds the column.
///
/// # Returns
///
/// * A Result that is either an empty Ok variant if the operation was successful, or an Err variant with an AppError.
pub async fn add_anime_to_global_kill_switch() -> Result<(), Box<dyn Error>> {
    let pool = get_postgresql_pool().await?;

    // Check if the "anime" column exists in the "global_kill_switch" table
    let row: (bool,) = sqlx::query_as(
        r#"
        SELECT EXISTS (
            SELECT  1
            FROM information_schema.columns
            WHERE table_name='global_kill_switch' AND column_name='anime'
        )
        "#,
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| error_enum::Error::Database(format!("Failed to execute query. {:#?}", e)))?;

    // If the "anime" column doesn't exist, add it
    if !row.0 {
        sqlx::query("ALTER TABLE global_kill_switch ADD COLUMN anime BIGINT")
            .execute(&pool)
            .await
            .map_err(|e| {
                error_enum::Error::Database(format!("Failed to execute query. {:#?}", e))
            })?;
    }

    pool.close().await;
    Ok(())
}

pub async fn add_vn_to_global_kill_switch() -> Result<(), Box<dyn Error>> {
    let pool = get_postgresql_pool().await?;

    // Check if the "vn" column exists in the "global_kill_switch" table
    let row: (bool,) = sqlx::query_as(
        r#"
        SELECT EXISTS (
            SELECT  1
            FROM information_schema.columns
            WHERE table_name='global_kill_switch' AND column_name='vn'
        )
        "#,
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| error_enum::Error::Database(format!("Failed to execute query. {:#?}", e)))?;

    // If the "vn" column doesn't exist, add it
    if !row.0 {
        sqlx::query("ALTER TABLE global_kill_switch ADD COLUMN vn BIGINT")
            .execute(&pool)
            .await
            .map_err(|e| {
                error_enum::Error::Database(format!("Failed to execute query. {:#?}", e))
            })?;
    }

    pool.close().await;
    Ok(())
}

pub async fn add_vn_to_module_activation() -> Result<(), Box<dyn Error>> {
    let pool = get_postgresql_pool().await?;

    // Check if the "vn" column exists in the "module_activation" table
    let row: (bool,) = sqlx::query_as(
        r#"
        SELECT EXISTS (
            SELECT  1
            FROM information_schema.columns
            WHERE table_name='module_activation' AND column_name='vn'
        )
        "#,
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| error_enum::Error::Database(format!("Failed to execute query. {:#?}", e)))?;

    // If the "vn" column doesn't exist, add it
    if !row.0 {
        sqlx::query("ALTER TABLE module_activation ADD COLUMN vn BIGINT")
            .execute(&pool)
            .await
            .map_err(|e| {
                error_enum::Error::Database(format!("Failed to execute query. {:#?}", e))
            })?;
    }

    pool.close().await;
    Ok(())
}
