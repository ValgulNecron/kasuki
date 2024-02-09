use crate::constant::DATA_SQLITE_DB;
use crate::database::sqlite::pool::get_sqlite_pool;
use crate::error_enum::AppError;
use crate::error_enum::AppError::NotACommandError;
use crate::error_enum::NotACommandError::{FailedToUpdateDatabase, SqlSelectError};

pub async fn migrate_sqlite() -> Result<(), AppError> {
    // used to update the database when new row are added to a table.
    add_image_to_activity_data().await?;
    add_new_member_to_module_activation().await?;
    add_new_member_to_global_kill_switch().await?;
    Ok(())
}

pub async fn add_image_to_activity_data() -> Result<(), AppError> {
    let pool = get_sqlite_pool(DATA_SQLITE_DB).await?;

    // Check if the "image" column exists in the "activity_data" table
    let row: u32 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM pragma_table_info('activity_data') WHERE name='image'",
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
    if row == 0 {
        sqlx::query("ALTER TABLE activity_data ADD COLUMN image TEXT DEFAULT 'data:image/png;base64,UklGRngFAABXRUJQVlA4IGwFAACQFQCdASpAAEAAPm0qkUWkIqGWDVeYQAbEoAvNeYDiDjvgYVI5A1hflD+HyFjgvqtG2WTHL+FrSd/NOhgzpPWXsEeWd7MPRV/cB30UGJsvLnbV1VqthWilezYCFAvetUFlshCk/xVa9IBRnL1Pp8HXWSJoXAGlB/F+74dS9N1/WmAgqV2g74G6aecKudBqp5vcYaiypIM+1wJSN29GOOiwmRrbfLgukEaogfmrmNBdDsfe9sQAAP7/toyQcQm08zUVT/Gcrfn+ngVdDMzXP4jQ9Hm0cRUTMClcf86TQ39SGSgXPW2HMB6sMZ2Vsv0/GOWT/WU+J+IfOI0Ai0aBHCQxDOZFaxrrdkA58PFTCDV9vJuUJkNTMqKBBoqRkica7jt1zj2Z23YDDhpCnNm00qBio5nLkVV0u83H3Qnbaz9kP3xd2llOCqg27pmLBEhtd3QJlmfoK+JMuIEhJgqOB5I1fXsnzBWQiz2mOvgSiwJAuuqAldkH4vt3085f8eCgoHu199XFVbU332orUfhK+G2vn7TQd/7VpAb0lxQDDyT00Zlxxy+O3/5IfIWIzJ1CTofiw6CL6Eew9xeAmtUoXAEW/ItApLeina9bCTKqiOfEVpnMHOBP+Rb9KMV1ugp6W4uYQ1/ontVdrBGxlXgbMI5otyujVdyUXd4P/PuS5i0Zkw6htI7E5sPsPowT5WAuADForLZb8lLSv8qGX4u4T+Pf4/qyH2nRzBg1CMokQIwSyeO27e5csfrjyx1x0AGcC/uB2N43gdNr/wAiud3isIiQVKYhBuckmRXUJZsILOKVPOdO37meUFDoNDGB9fJoOCcEvwYNIebfA/5rlXItCzE4ah0kXDWP5GWbpq7dNyE+6GFbo8IHcg5IEgvtwM3J54BYBkADUJD9C+VHBAPzEQnv0e3yW0q6xgMDawUKfPHaY5vXK/uShArGmWwvhZwXMYtWnB+j2QiRMVC2PY/jlZYuEjafnGcFZdelBZWXGd8ewCyUVBl/LZYjEhcs7mGCj4VtsfP6onACpogMRlr8j302Jdt/Gf+XqbC+3wUmLVgKvrmWwve+yMEeI7IsmR1xLMZlGPmfvoohtoMYp3J3ogXMa66etFb2L3L+y3pvb1rh08nXxNS40gEYXWNb2+3Dcr4iHiAuP8FkMF0snZEPOz/3EOn0IGnmqQ7S34g48tTUJylINg3uXOeRZgHLncuThPa1Igb+ZdZ79ndbQ0LBUemYhtbD9KKXPDhnFvxMwOBPgOyB/rjrJ5eJ9/O8V5EhKBRtcAaHDBiyDUVGIz7EKZ/FT39pP/XtwKiVnDmd69AoGHBeeyhwgDV7rVbwB0lQW3xwddCfiA/vGqWypJ6l6c0DfPSw3dgQv5bZMKLWjQyD6EZBPDOCi2p0iQsgD7wlIxZylSxYJlfNxJbw9l/PT0mQk9CdNwQYUvaXKbOrSYjfNpkFE6Tae6gURO2X3M+sPV0UQr3IuiD06ur5wGUQrXK/Ldl+JopFMeQb/8wvMgcKiwmrmQf6MaWcZnCb4CFbf1xAxG2KK2m/H47PHg8RYJH/X+8fb6hqBLq7DWKxUlAuB25UL0MA+oXZwahI8s4AeTjm65G8m5PS2Z+VBcYwsUHDK6pbcFMul9CbIQBtX/HLHetzai1n+MielaSfiY78EY8ZMAuZXYD/xC/b0NZyQDQ76fOokEUdTzU5ev2QdJMtI1UEtE+6toA8jPzy568UDnSvfZLCr2Rj4IP5bXMUhTCumOalI/4kAGmughKGrp+dm1MPN3Tg/oJPXy//yNYw9bton7goPbN5nyg0Yjfyr4Qq6V+twPgYTbRIC+fHeH0/agtR3nWQYE8PEAAAAA=='")
            .execute(&pool)
            .await
            .map_err(|e| NotACommandError(FailedToUpdateDatabase(format!("Failed to update the table. {}", e))))?;
    }

    pool.close().await;
    Ok(())
}

pub async fn add_new_member_to_module_activation() -> Result<(), AppError> {
    let pool = get_sqlite_pool(DATA_SQLITE_DB).await?;

    // Check if the "new_member" column exists in the "module_activation" table
    let row: u32 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM pragma_table_info('module_activation') WHERE name='new_member'",
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        NotACommandError(SqlSelectError(format!(
            "Failed to select from the table. {}",
            e
        )))
    })?;

    // If the "new_member" column doesn't exist, add it
    if row == 0 {
        sqlx::query("ALTER TABLE module_activation ADD COLUMN new_member TEXT")
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

pub async fn add_new_member_to_global_kill_switch() -> Result<(), AppError> {
    let pool = get_sqlite_pool(DATA_SQLITE_DB).await?;

    // Check if the "new_member" column exists in the "global_kill_switch" table
    let row: u32 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM pragma_table_info('global_kill_switch') WHERE name='new_member'",
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        NotACommandError(SqlSelectError(format!(
            "Failed to select from the table. {}",
            e
        )))
    })?;

    // If the "new_member" column doesn't exist, add it
    if row == 0 {
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
