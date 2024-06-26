use crate::database::data_struct::guild_language::GuildLanguage;
use crate::database::data_struct::module_status::ActivationStatusModule;
use crate::database::data_struct::ping_history::PingHistory;
use crate::database::data_struct::registered_user::RegisteredUser;
use crate::database::data_struct::server_activity::{
    ServerActivity, ServerActivityFull, SmallServerActivity,
};
use crate::database::data_struct::user_color::UserColor;
use crate::database::manage::postgresql::data::{
    get_all_server_activity_postgresql, get_all_user_approximated_color_postgres,
    get_data_activity_postgresql, get_data_all_activity_by_server_postgresql,
    get_data_guild_language_postgresql, get_data_module_activation_kill_switch_status_postgresql,
    get_data_module_activation_status_postgresql, get_one_activity_postgresql,
    get_registered_user_postgresql, get_server_image_postgresql,
    get_user_approximated_color_postgresql, remove_data_activity_status_postgresql,
    set_data_activity_postgresql, set_data_guild_language_postgresql,
    set_data_module_activation_status_postgresql, set_data_ping_history_postgresql,
    set_registered_user_postgresql, set_server_image_postgresql,
    set_user_approximated_color_postgresql,
};
use crate::database::manage::sqlite::data::{
    get_all_server_activity_sqlite, get_all_user_approximated_color_sqlite,
    get_data_activity_sqlite, get_data_all_activity_by_server_sqlite,
    get_data_guild_language_sqlite, get_data_module_activation_kill_switch_status_sqlite,
    get_data_module_activation_status_sqlite, get_one_activity_sqlite, get_registered_user_sqlite,
    get_server_image_sqlite, get_user_approximated_color_sqlite,
    remove_data_activity_status_sqlite, set_data_activity_sqlite, set_data_guild_language_sqlite,
    set_data_module_activation_status_sqlite, set_data_ping_history_sqlite,
    set_registered_user_sqlite, set_server_image_sqlite, set_user_approximated_color_sqlite,
};
use crate::helper::error_management::error_enum::AppError;

/// Sets the ping history in the database.
///
/// This function takes a shard ID and a latency as parameters.
/// It retrieves the type of the database from the environment variables and defaults to "sqlite" if it is not set.
/// Depending on the type of the database, it calls the corresponding function to set the ping history.
///
/// # Arguments
///
/// * `shard_id` - A string that represents the shard ID.
/// * `latency` - A string that represents the latency.
///
/// # Returns
///
/// * A Result that is either an empty Ok variant if the operation was successful, or an Err variant with an AppError.
pub async fn set_data_ping_history(
    ping_history: PingHistory,
    db_type: String,
) -> Result<(), AppError> {
    if db_type == "sqlite" {
        set_data_ping_history_sqlite(ping_history).await
    } else if db_type == "postgresql" {
        set_data_ping_history_postgresql(ping_history).await
    } else {
        set_data_ping_history_sqlite(ping_history).await
    }
}

/// Retrieves the guild language from the database.
///
/// This function takes a guild ID as a parameter.
/// It retrieves the type of the database from the environment variables and defaults to "sqlite" if it is not set.
/// Depending on the type of the database, it calls the corresponding function to get the guild language.
///
/// # Arguments
///
/// * `guild_id` - A string that represents the guild ID.
///
/// # Returns
///
/// * A Result that is either an Option variant containing the guild language if the operation was successful, or an Err variant with an AppError.
pub async fn get_data_guild_language(
    guild_id: String,
    db_type: String,
) -> Result<Option<GuildLanguage>, AppError> {
    let db_type = db_type.as_str();
    if db_type == "sqlite" {
        get_data_guild_language_sqlite(guild_id).await
    } else if db_type == "postgresql" {
        get_data_guild_language_postgresql(guild_id).await
    } else {
        get_data_guild_language_sqlite(guild_id).await
    }
}

/// Sets the guild language in the database.
///
/// This function takes a guild ID and a language as parameters.
/// It retrieves the type of the database from the environment variables and defaults to "sqlite" if it is not set.
/// Depending on the type of the database, it calls the corresponding function to set the guild language.
///
/// # Arguments
///
/// * `guild_id` - A string that represents the guild ID.
/// * `lang` - A string that represents the language.
///
/// # Returns
///
/// * A Result that is either an empty Ok variant if the operation was successful, or an Err variant with an AppError.
pub async fn set_data_guild_language(
    guild_language: GuildLanguage,
    db_type: &str,
) -> Result<(), AppError> {
    if db_type == "sqlite" {
        set_data_guild_language_sqlite(guild_language).await
    } else if db_type == "postgresql" {
        set_data_guild_language_postgresql(guild_language).await
    } else {
        set_data_guild_language_sqlite(guild_language).await
    }
}

/// Retrieves the activity data from the database.
///
/// This function takes a timestamp as a parameter.
/// It retrieves the type of the database from the environment variables and defaults to "sqlite" if it is not set.
/// Depending on the type of the database, it calls the corresponding function to get the activity data.
///
/// # Arguments
///
/// * `now` - A string that represents the current timestamp.
///
/// # Returns
///
/// * A Result that is either a Vec variant containing the activity data if the operation was successful, or an Err variant with an AppError.
pub async fn get_data_activity(
    now: String,
    db_type: String,
) -> Result<Vec<ServerActivityFull>, AppError> {
    let db_type = db_type.as_str();
    if db_type == "sqlite" {
        get_data_activity_sqlite(now).await
    } else if db_type == "postgresql" {
        get_data_activity_postgresql(now).await
    } else {
        get_data_activity_sqlite(now).await
    }
}

/// Sets the activity data in the database.
///
/// This function takes a ServerActivityFull object as a parameter.
/// It retrieves the type of the database from the environment variables and defaults to "sqlite" if it is not set.
/// Depending on the type of the database, it calls the corresponding function to set the activity data.
///
/// # Arguments
///
/// * `server_activity_full` - A ServerActivityFull object that represents the activity data.
///
/// # Returns
///
/// * A Result that is either an empty Ok variant if the operation was successful, or an Err variant with an AppError.
pub async fn set_data_activity(
    server_activity_full: ServerActivityFull,
    db_type: String,
) -> Result<(), AppError> {
    let db_type = db_type.as_str();
    if db_type == "sqlite" {
        set_data_activity_sqlite(server_activity_full).await
    } else if db_type == "postgresql" {
        set_data_activity_postgresql(server_activity_full).await
    } else {
        set_data_activity_sqlite(server_activity_full).await
    }
}

/// Retrieves the module activation status from the database.
///
/// This function takes a guild ID as a parameter.
/// It retrieves the type of the database from the environment variables and defaults to "sqlite" if it is not set.
/// Depending on the type of the database, it calls the corresponding function to get the module activation status.
///
/// # Arguments
///
/// * `guild_id` - A string that represents the guild ID.
///
/// # Returns
///
/// * A Result that is either an ActivationStatusModule variant if the operation was successful, or an Err variant with an AppError.
pub async fn get_data_module_activation_status(
    guild_id: &String,
    db_type: &str,
) -> Result<ActivationStatusModule, AppError> {
    if db_type == "sqlite" {
        get_data_module_activation_status_sqlite(guild_id).await
    } else if db_type == "postgresql" {
        get_data_module_activation_status_postgresql(guild_id).await
    } else {
        get_data_module_activation_status_sqlite(guild_id).await
    }
}

/// Sets the module activation status in the database.
///
/// This function takes a guild ID and the activation status of various modules as parameters.
/// It retrieves the type of the database from the environment variables and defaults to "sqlite" if it is not set.
/// Depending on the type of the database, it calls the corresponding function to set the module activation status.
///
/// # Arguments
///
/// * `guild_id` - A string that represents the guild ID.
/// * `anilist_value` - A boolean that represents the activation status of the Anilist module.
/// * `ai_value` - A boolean that represents the activation status of the AI module.
/// * `game_value` - A boolean that represents the activation status of the Game module.
/// * `new_member_value` - A boolean that represents the activation status of the New Member module.
///
/// # Returns
///
/// * A Result that is either an empty Ok variant if the operation was successful, or an Err variant with an AppError.
pub async fn set_data_module_activation_status(
    activation_status_module: ActivationStatusModule,
    db_type: &str,
) -> Result<(), AppError> {
    if db_type == "sqlite" {
        set_data_module_activation_status_sqlite(activation_status_module).await
    } else if db_type == "postgresql" {
        set_data_module_activation_status_postgresql(activation_status_module).await
    } else {
        set_data_module_activation_status_sqlite(activation_status_module).await
    }
}

/// Retrieves a specific activity from the database.
///
/// This function takes an anime ID and a server ID as parameters.
/// It retrieves the type of the database from the environment variables and defaults to "sqlite" if it is not set.
/// Depending on the type of the database, it calls the corresponding function to get the activity.
///
/// # Arguments
///
/// * `anime_id` - An i32 that represents the anime ID.
/// * `server_id` - A string that represents the server ID.
///
/// # Returns
///
/// * A Result that is either a tuple containing the Option variants of the activity if the operation was successful, or an Err variant with an AppError.
pub async fn get_one_activity(
    anime_id: i32,
    server_id: String,
    db_type: &str,
) -> Result<SmallServerActivity, AppError> {
    if db_type == "sqlite" {
        get_one_activity_sqlite(server_id, anime_id).await
    } else if db_type == "postgresql" {
        get_one_activity_postgresql(server_id, anime_id).await
    } else {
        get_one_activity_sqlite(server_id, anime_id).await
    }
}

/// Retrieves a registered user from the database.
///
/// This function takes a user ID as a parameter.
/// It retrieves the type of the database from the environment variables and defaults to "sqlite" if it is not set.
/// Depending on the type of the database, it calls the corresponding function to get the registered user.
///
/// # Arguments
///
/// * `user_id` - A string that represents the user ID.
///
/// # Returns
///
/// * A Result that is either a tuple containing the Option variants of the registered user if the operation was successful, or an Err variant with an AppError.
pub async fn get_registered_user(
    user_id: &String,
    db_type: &str,
) -> Result<Option<RegisteredUser>, AppError> {
    if db_type == "sqlite" {
        get_registered_user_sqlite(user_id).await
    } else if db_type == "postgresql" {
        get_registered_user_postgresql(user_id).await
    } else {
        get_registered_user_sqlite(user_id).await
    }
}

/// Registers a user in the database.
///
/// This function takes a user ID and a username as parameters.
/// It retrieves the type of the database from the environment variables and defaults to "sqlite" if it is not set.
/// Depending on the type of the database, it calls the corresponding function to register the user.
///
/// # Arguments
///
/// * `user_id` - A string that represents the user ID.
/// * `username` - A string that represents the username.
///
/// # Returns
///
/// * A Result that is either an empty Ok variant if the operation was successful, or an Err variant with an AppError.
pub async fn set_registered_user(
    registered_user: RegisteredUser,
    db_type: &str,
) -> Result<(), AppError> {
    if db_type == "sqlite" {
        set_registered_user_sqlite(registered_user).await
    } else if db_type == "postgresql" {
        set_registered_user_postgresql(registered_user).await
    } else {
        set_registered_user_sqlite(registered_user).await
    }
}

/// Retrieves the module activation kill switch status from the database.
///
/// This function does not take any parameters.
/// It retrieves the type of the database from the environment variables and defaults to "sqlite" if it is not set.
/// Depending on the type of the database, it calls the corresponding function to get the module activation kill switch status.
///
/// # Returns
///
/// * A Result that is either an ActivationStatusModule variant if the operation was successful, or an Err variant with an AppError.
pub async fn get_data_module_activation_kill_switch_status(
    db_type: &str,
) -> Result<ActivationStatusModule, AppError> {
    if db_type == "sqlite" {
        get_data_module_activation_kill_switch_status_sqlite().await
    } else if db_type == "postgresql" {
        get_data_module_activation_kill_switch_status_postgresql().await
    } else {
        get_data_module_activation_kill_switch_status_sqlite().await
    }
}

/// Removes an activity status from the database.
///
/// This function takes a server ID and an anime ID as parameters.
/// It retrieves the type of the database from the environment variables and defaults to "sqlite" if it is not set.
/// Depending on the type of the database, it calls the corresponding function to remove the activity status.
///
/// # Arguments
///
/// * `server_id` - A string that represents the server ID.
/// * `anime_id` - A string that represents the anime ID.
///
/// # Returns
///
/// * A Result that is either an empty Ok variant if the operation was successful, or an Err variant with an AppError.
pub async fn remove_data_activity_status(
    server_id: String,
    anime_id: String,
    db_type: String,
) -> Result<(), AppError> {
    let db_type = db_type.as_str();
    if db_type == "sqlite" {
        remove_data_activity_status_sqlite(server_id, anime_id).await
    } else if db_type == "postgresql" {
        remove_data_activity_status_postgresql(server_id, anime_id).await
    } else {
        remove_data_activity_status_sqlite(server_id, anime_id).await
    }
}

/// Sets the approximated color for a user in the database.
///
/// This function takes a user ID, color, profile picture URL, and image as parameters.
/// It retrieves the type of the database from the environment variables and defaults to "sqlite" if it is not set.
/// Depending on the type of the database, it calls the corresponding function to set the user's approximated color.
///
/// # Arguments
///
/// * `user_id` - A string that represents the user ID.
/// * `color` - A string that represents the approximated color.
/// * `pfp_url` - A string that represents the URL of the user's profile picture.
/// * `image` - A string that represents the image.
///
/// # Returns
///
/// * A Result that is either an empty Ok variant if the operation was successful, or an Err variant with an AppError.
pub async fn set_user_approximated_color(
    user_id: &String,
    color: &String,
    pfp_url: &String,
    image: &String,
    db_type: String,
) -> Result<(), AppError> {
    let db_type = db_type.as_str();
    if db_type == "sqlite" {
        set_user_approximated_color_sqlite(user_id, color, pfp_url, image).await
    } else if db_type == "postgresql" {
        set_user_approximated_color_postgresql(user_id, color, pfp_url, image).await
    } else {
        set_user_approximated_color_sqlite(user_id, color, pfp_url, image).await
    }
}

/// Retrieves the approximated color for a user from the database.
///
/// This function takes a user ID as a parameter.
/// It retrieves the type of the database from the environment variables and defaults to "sqlite" if it is not set.
/// Depending on the type of the database, it calls the corresponding function to get the user's approximated color.
///
/// # Arguments
///
/// * `user_id` - A string that represents the user ID.
///
/// # Returns
///
/// * A Result that is either a UserColor variant if the operation was successful, or an Err variant with an AppError.
pub async fn get_user_approximated_color(
    user_id: &String,
    db_type: String,
) -> Result<UserColor, AppError> {
    let db_type = db_type.as_str();
    if db_type == "sqlite" {
        get_user_approximated_color_sqlite(user_id).await
    } else if db_type == "postgresql" {
        get_user_approximated_color_postgresql(user_id).await
    } else {
        get_user_approximated_color_sqlite(user_id).await
    }
}

/// Retrieves all server activity from the database.
///
/// This function takes a server ID as a parameter.
/// It retrieves the type of the database from the environment variables and defaults to "sqlite" if it is not set.
/// Depending on the type of the database, it calls the corresponding function to get all server activity.
///
/// # Arguments
///
/// * `server_id` - A string that represents the server ID.
///
/// # Returns
///
/// * A Result that is either a Vec variant containing the ServerActivity if the operation was successful, or an Err variant with an AppError.
pub async fn get_all_server_activity(
    server_id: &String,
    db_type: String,
) -> Result<Vec<ServerActivity>, AppError> {
    let db_type = db_type.as_str();
    if db_type == "sqlite" {
        get_all_server_activity_sqlite(server_id).await
    } else if db_type == "postgresql" {
        get_all_server_activity_postgresql(server_id).await
    } else {
        get_all_server_activity_sqlite(server_id).await
    }
}

/// Retrieves all activity by server from the database.
///
/// This function takes a server ID as a parameter.
/// It retrieves the type of the database from the environment variables and defaults to "sqlite" if it is not set.
/// Depending on the type of the database, it calls the corresponding function to get all activity by server.
///
/// # Arguments
///
/// * `server_id` - A string that represents the server ID.
///
/// # Returns
///
/// * A Result that is either a Vec variant containing tuples of String if the operation was successful, or an Err variant with an AppError.
pub async fn get_data_all_activity_by_server(
    server_id: &String,
    db_type: &str,
) -> Result<Vec<(String, String)>, AppError> {
    if db_type == "sqlite" {
        get_data_all_activity_by_server_sqlite(server_id).await
    } else if db_type == "postgresql" {
        get_data_all_activity_by_server_postgresql(server_id).await
    } else {
        get_data_all_activity_by_server_sqlite(server_id).await
    }
}

/// Retrieves all user approximated colors from the database.
///
/// This function does not take any parameters.
/// It retrieves the type of the database from the environment variables and defaults to "sqlite" if it is not set.
/// Depending on the type of the database, it calls the corresponding function to get all user approximated colors.
///
/// # Returns
///
/// * A Result that is either a Vec variant containing UserColor if the operation was successful, or an Err variant with an AppError.
pub async fn get_all_user_approximated_color(db_type: &str) -> Result<Vec<UserColor>, AppError> {
    if db_type == "sqlite" {
        get_all_user_approximated_color_sqlite().await
    } else if db_type == "postgresql" {
        get_all_user_approximated_color_postgres().await
    } else {
        get_all_user_approximated_color_sqlite().await
    }
}

/// Sets the server image in the database.
///
/// This function takes a server ID, image type, image, and image URL as parameters.
/// It retrieves the type of the database from the environment variables and defaults to "sqlite" if it is not set.
/// Depending on the type of the database, it calls the corresponding function to set the server image.
///
/// # Arguments
///
/// * `server_id` - A string that represents the server ID.
/// * `image_type` - A string that represents the type of the image.
/// * `image` - A string that represents the image.
/// * `image_url` - A string that represents the URL of the image.
///
/// # Returns
///
/// * A Result that is either an empty Ok variant if the operation was successful, or an Err variant with an AppError.
pub async fn set_server_image(
    server_id: &String,
    image_type: &String,
    image: &String,
    image_url: &String,
    db_type: &str,
) -> Result<(), AppError> {
    if db_type == "sqlite" {
        set_server_image_sqlite(server_id, image_type, image, image_url).await
    } else if db_type == "postgresql" {
        set_server_image_postgresql(server_id, image_type, image, image_url).await
    } else {
        set_server_image_sqlite(server_id, image_type, image, image_url).await
    }
}

/// Retrieves the server image from the database.
///
/// This function takes a server ID and an image type as parameters.
/// It retrieves the type of the database from the environment variables and defaults to "sqlite" if it is not set.
/// Depending on the type of the database, it calls the corresponding function to get the server image.
///
/// # Arguments
///
/// * `server_id` - A string that represents the server ID.
/// * `image_type` - A string that represents the type of the image.
///
/// # Returns
///
/// * A Result that is either a tuple containing the Option variants of the image and its URL if the operation was successful, or an Err variant with an AppError.
pub async fn get_server_image(
    server_id: &String,
    image_type: &String,
    db_type: &str,
) -> Result<(Option<String>, Option<String>), AppError> {
    if db_type == "sqlite" {
        get_server_image_sqlite(server_id, image_type).await
    } else if db_type == "postgresql" {
        get_server_image_postgresql(server_id, image_type).await
    } else {
        get_server_image_sqlite(server_id, image_type).await
    }
}
