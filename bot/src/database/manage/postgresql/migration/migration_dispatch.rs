use crate::config::BotConfigDetails;
use crate::database::manage::postgresql::pool::get_postgresql_pool;
use std::error::Error;
use tracing::warn;

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
pub async fn migrate_postgres(db_config: BotConfigDetails) -> Result<(), Box<dyn Error>> {
    // used to update the database when new row are added to a table.
    add_image_to_activity_data(db_config.clone()).await?;
    add_new_member_to_global_kill_switch(db_config.clone()).await?;
    add_new_member_to_module_activation(db_config.clone()).await?;
    add_anime_to_global_kill_switch(db_config.clone()).await?;
    add_anime_to_module_activation(db_config.clone()).await?;
    add_vn_to_global_kill_switch(db_config.clone()).await?;
    add_vn_to_module_activation(db_config.clone()).await?;
    update_name_of_id_in_global_kill_switch(db_config.clone()).await?;
    update_name_of_id_in_global_module_activation(db_config).await?;
    Ok(())
}

async fn update_name_of_id_in_global_kill_switch(
    db_config: BotConfigDetails,
) -> Result<(), Box<dyn Error>> {
    // change the name of the row id to guild_id
    let pool = get_postgresql_pool(db_config).await?;

    // if in kasuki.global_kill_switch the columm name is "id" rename it to "guild_id"
    match sqlx::query("ALTER TABLE global_kill_switch RENAME COLUMN id TO guild_id")
        .execute(&pool)
        .await
    {
        Ok(_) => (),
        Err(e) => {
            warn!("Failed to execute query. {:#?}", e);
        }
    };
    Ok(())
}
async fn update_name_of_id_in_global_module_activation(
    db_config: BotConfigDetails,
) -> Result<(), Box<dyn Error>> {
    // change the name of the row id to guild_id
    let pool = get_postgresql_pool(db_config).await?;
    match sqlx::query("ALTER TABLE module_activation RENAME COLUMN id TO guild_id")
        .execute(&pool)
        .await
    {
        Ok(_) => (),
        Err(e) => {
            warn!("Failed to execute query. {:#?}", e);
        }
    };
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
pub async fn add_image_to_activity_data(db_config: BotConfigDetails) -> Result<(), Box<dyn Error>> {
    let pool = get_postgresql_pool(db_config).await?;

    match sqlx::query("ALTER TABLE activity_data ADD COLUMN image TEXT")
        .execute(&pool)
        .await
    {
        Ok(_) => (),
        Err(e) => {
            warn!("Failed to execute query. {:#?}", e);
        }
    }

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
pub async fn add_new_member_to_global_kill_switch(
    db_config: BotConfigDetails,
) -> Result<(), Box<dyn Error>> {
    let pool = get_postgresql_pool(db_config).await?;

    match sqlx::query("ALTER TABLE global_kill_switch ADD COLUMN new_member BIGINT")
        .execute(&pool)
        .await
    {
        Ok(_) => (),
        Err(e) => {
            warn!("Failed to execute query. {:#?}", e);
        }
    }

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
pub async fn add_new_member_to_module_activation(
    db_config: BotConfigDetails,
) -> Result<(), Box<dyn Error>> {
    let pool = get_postgresql_pool(db_config).await?;

    match sqlx::query("ALTER TABLE module_activation ADD COLUMN new_member BIGINT")
        .execute(&pool)
        .await
    {
        Ok(_) => (),
        Err(e) => {
            warn!("Failed to execute query. {:#?}", e);
        }
    }

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
pub async fn add_anime_to_module_activation(
    db_config: BotConfigDetails,
) -> Result<(), Box<dyn Error>> {
    let pool = get_postgresql_pool(db_config).await?;

    match sqlx::query("ALTER TABLE module_activation ADD COLUMN anime BIGINT")
        .execute(&pool)
        .await
    {
        Ok(_) => (),
        Err(e) => {
            warn!("Failed to execute query. {:#?}", e);
        }
    }

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
pub async fn add_anime_to_global_kill_switch(
    db_config: BotConfigDetails,
) -> Result<(), Box<dyn Error>> {
    let pool = get_postgresql_pool(db_config).await?;

    match sqlx::query("ALTER TABLE global_kill_switch ADD COLUMN anime BIGINT")
        .execute(&pool)
        .await
    {
        Ok(_) => (),
        Err(e) => {
            warn!("Failed to execute query. {:#?}", e);
        }
    }

    Ok(())
}

pub async fn add_vn_to_global_kill_switch(
    db_config: BotConfigDetails,
) -> Result<(), Box<dyn Error>> {
    let pool = get_postgresql_pool(db_config).await?;

    match sqlx::query("ALTER TABLE global_kill_switch ADD COLUMN vn BIGINT")
        .execute(&pool)
        .await
    {
        Ok(_) => (),
        Err(e) => {
            warn!("Failed to execute query. {:#?}", e);
        }
    }

    Ok(())
}

pub async fn add_vn_to_module_activation(
    db_config: BotConfigDetails,
) -> Result<(), Box<dyn Error>> {
    let pool = get_postgresql_pool(db_config).await?;

    match sqlx::query("ALTER TABLE module_activation ADD COLUMN vn BIGINT")
        .execute(&pool)
        .await
    {
        Ok(_) => (),
        Err(e) => {
            warn!("Failed to execute query. {:#?}", e);
        }
    }

    Ok(())
}
