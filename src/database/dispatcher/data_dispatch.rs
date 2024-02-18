use std::env;

use crate::anilist_struct::run::minimal_anime::ActivityData;
use crate::database::postgresql::data::{
    get_all_server_activity_postgresql, get_all_user_approximated_color_postgres,
    get_data_activity_postgresql, get_data_activity_with_server_and_anime_id_postgresql,
    get_data_all_activity_by_server_postgresql, get_data_guild_language_postgresql,
    get_data_module_activation_kill_switch_status_postgresql,
    get_data_module_activation_status_postgresql, get_one_activity_postgresql,
    get_registered_user_postgresql, get_server_image_postgresql,
    get_user_approximated_color_postgresql, remove_data_activity_status_postgresql,
    set_data_activity_postgresql, set_data_guild_language_postgresql,
    set_data_module_activation_status_postgresql, set_data_ping_history_postgresql,
    set_registered_user_postgresql, set_server_image_postgresql,
    set_user_approximated_color_postgresql,
};
use crate::database::sqlite::data::{
    get_all_server_activity_sqlite, get_all_user_approximated_color_sqlite,
    get_data_activity_sqlite, get_data_activity_with_server_and_anime_id_sqlite,
    get_data_all_activity_by_server_sqlite, get_data_guild_language_sqlite,
    get_data_module_activation_kill_switch_status_sqlite, get_data_module_activation_status_sqlite,
    get_one_activity_sqlite, get_registered_user_sqlite, get_server_image_sqlite,
    get_user_approximated_color_sqlite, remove_data_activity_status_sqlite,
    set_data_activity_sqlite, set_data_guild_language_sqlite,
    set_data_module_activation_status_sqlite, set_data_ping_history_sqlite,
    set_registered_user_sqlite, set_server_image_sqlite, set_user_approximated_color_sqlite,
};
use crate::database_struct::module_status::ActivationStatusModule;
use crate::database_struct::server_activity_struct::{ServerActivity, ServerActivityFull};
use crate::database_struct::user_color_struct::UserColor;
use crate::error_management::error_enum::AppError;

pub async fn set_data_ping_history(shard_id: String, latency: String) -> Result<(), AppError> {
    let db_type = env::var("DB_TYPE").unwrap_or("sqlite".to_string());
    if db_type == *"sqlite" {
        set_data_ping_history_sqlite(shard_id, latency).await
    } else if db_type == *"postgresql" {
        set_data_ping_history_postgresql(shard_id, latency).await
    } else {
        set_data_ping_history_sqlite(shard_id, latency).await
    }
}

pub async fn get_data_guild_language(
    guild_id: String,
) -> Result<(Option<String>, Option<String>), AppError> {
    let db_type = env::var("DB_TYPE").unwrap_or("sqlite".to_string());
    if db_type == *"sqlite" {
        get_data_guild_language_sqlite(guild_id).await
    } else if db_type == *"postgresql" {
        get_data_guild_language_postgresql(guild_id).await
    } else {
        get_data_guild_language_sqlite(guild_id).await
    }
}

pub async fn set_data_guild_language(guild_id: &String, lang: &String) -> Result<(), AppError> {
    let db_type = env::var("DB_TYPE").unwrap_or("sqlite".to_string());
    if db_type == *"sqlite" {
        set_data_guild_language_sqlite(guild_id, lang).await
    } else if db_type == *"postgresql" {
        set_data_guild_language_postgresql(guild_id, lang).await
    } else {
        set_data_guild_language_sqlite(guild_id, lang).await
    }
}

pub async fn get_data_activity(now: String) -> Result<Vec<ActivityData>, AppError> {
    let db_type = env::var("DB_TYPE").unwrap_or("sqlite".to_string());
    if db_type == *"sqlite" {
        get_data_activity_sqlite(now).await
    } else if db_type == *"postgresql" {
        get_data_activity_postgresql(now).await
    } else {
        get_data_activity_sqlite(now).await
    }
}

pub async fn set_data_activity(server_activity_full: ServerActivityFull) -> Result<(), AppError> {
    let db_type = env::var("DB_TYPE").unwrap_or("sqlite".to_string());
    if db_type == *"sqlite" {
        set_data_activity_sqlite(server_activity_full).await
    } else if db_type == *"postgresql" {
        set_data_activity_postgresql(server_activity_full).await
    } else {
        set_data_activity_sqlite(server_activity_full).await
    }
}

pub async fn get_data_module_activation_status(
    guild_id: &String,
) -> Result<ActivationStatusModule, AppError> {
    let db_type = env::var("DB_TYPE").unwrap_or("sqlite".to_string());
    if db_type == *"sqlite" {
        get_data_module_activation_status_sqlite(guild_id).await
    } else if db_type == *"postgresql" {
        get_data_module_activation_status_postgresql(guild_id).await
    } else {
        get_data_module_activation_status_sqlite(guild_id).await
    }
}

pub async fn set_data_module_activation_status(
    guild_id: &String,
    anilist_value: bool,
    ai_value: bool,
    game_value: bool,
    new_member_value: bool,
) -> Result<(), AppError> {
    let db_type = env::var("DB_TYPE").unwrap_or("sqlite".to_string());
    if db_type == *"sqlite" {
        set_data_module_activation_status_sqlite(
            guild_id,
            anilist_value,
            ai_value,
            game_value,
            new_member_value,
        )
        .await
    } else if db_type == *"postgresql" {
        set_data_module_activation_status_postgresql(
            guild_id,
            anilist_value,
            ai_value,
            game_value,
            new_member_value,
        )
        .await
    } else {
        set_data_module_activation_status_sqlite(
            guild_id,
            anilist_value,
            ai_value,
            game_value,
            new_member_value,
        )
        .await
    }
}

pub async fn get_one_activity(
    anime_id: i32,
    server_id: String,
) -> Result<(Option<String>, Option<String>, Option<String>), AppError> {
    let db_type = env::var("DB_TYPE").unwrap_or("sqlite".to_string());
    if db_type == *"sqlite" {
        get_one_activity_sqlite(server_id, anime_id).await
    } else if db_type == *"postgresql" {
        get_one_activity_postgresql(server_id, anime_id).await
    } else {
        get_one_activity_sqlite(server_id, anime_id).await
    }
}

pub async fn get_registered_user(
    user_id: &String,
) -> Result<(Option<String>, Option<String>), AppError> {
    let db_type = env::var("DB_TYPE").unwrap_or("sqlite".to_string());
    if db_type == *"sqlite" {
        get_registered_user_sqlite(user_id).await
    } else if db_type == *"postgresql" {
        get_registered_user_postgresql(user_id).await
    } else {
        get_registered_user_sqlite(user_id).await
    }
}

pub async fn set_registered_user(user_id: &String, username: &String) -> Result<(), AppError> {
    let db_type = env::var("DB_TYPE").unwrap_or("sqlite".to_string());
    if db_type == *"sqlite" {
        set_registered_user_sqlite(user_id, username).await
    } else if db_type == *"postgresql" {
        set_registered_user_postgresql(user_id, username).await
    } else {
        set_registered_user_sqlite(user_id, username).await
    }
}

pub async fn get_data_module_activation_kill_switch_status(
) -> Result<ActivationStatusModule, AppError> {
    let db_type = env::var("DB_TYPE").unwrap_or("sqlite".to_string());
    if db_type == *"sqlite" {
        get_data_module_activation_kill_switch_status_sqlite().await
    } else if db_type == *"postgresql" {
        get_data_module_activation_kill_switch_status_postgresql().await
    } else {
        get_data_module_activation_kill_switch_status_sqlite().await
    }
}

pub async fn remove_data_activity_status(
    server_id: String,
    anime_id: String,
) -> Result<(), AppError> {
    let db_type = env::var("DB_TYPE").unwrap_or("sqlite".to_string());
    if db_type == *"sqlite" {
        remove_data_activity_status_sqlite(server_id, anime_id).await
    } else if db_type == *"postgresql" {
        remove_data_activity_status_postgresql(server_id, anime_id).await
    } else {
        remove_data_activity_status_sqlite(server_id, anime_id).await
    }
}

pub async fn set_user_approximated_color(
    user_id: &String,
    color: &String,
    pfp_url: &String,
    image: &String,
) -> Result<(), AppError> {
    let db_type = env::var("DB_TYPE").unwrap_or("sqlite".to_string());
    if db_type == *"sqlite" {
        set_user_approximated_color_sqlite(user_id, color, pfp_url, image).await
    } else if db_type == *"postgresql" {
        set_user_approximated_color_postgresql(user_id, color, pfp_url, image).await
    } else {
        set_user_approximated_color_sqlite(user_id, color, pfp_url, image).await
    }
}

pub async fn get_user_approximated_color(user_id: &String) -> Result<UserColor, AppError> {
    let db_type = env::var("DB_TYPE").unwrap_or("sqlite".to_string());
    if db_type == *"sqlite" {
        get_user_approximated_color_sqlite(user_id).await
    } else if db_type == *"postgresql" {
        get_user_approximated_color_postgresql(user_id).await
    } else {
        get_user_approximated_color_sqlite(user_id).await
    }
}

pub async fn get_all_server_activity(server_id: &String) -> Result<Vec<ServerActivity>, AppError> {
    let db_type = env::var("DB_TYPE").unwrap_or("sqlite".to_string());
    if db_type == *"sqlite" {
        get_all_server_activity_sqlite(server_id).await
    } else if db_type == *"postgresql" {
        get_all_server_activity_postgresql(server_id).await
    } else {
        get_all_server_activity_sqlite(server_id).await
    }
}

pub async fn get_data_activity_with_server_and_anime_id(
    anime_id: &String,
    server_id: &String,
) -> Result<Option<String>, AppError> {
    let db_type = env::var("DB_TYPE").unwrap_or("sqlite".to_string());
    if db_type == *"sqlite" {
        get_data_activity_with_server_and_anime_id_sqlite(anime_id, server_id).await
    } else if db_type == *"postgresql" {
        get_data_activity_with_server_and_anime_id_postgresql(anime_id, server_id).await
    } else {
        get_data_activity_with_server_and_anime_id_sqlite(anime_id, server_id).await
    }
}

pub async fn get_data_all_activity_by_server(
    server_id: &String,
) -> Result<Vec<(String, String)>, AppError> {
    let db_type = env::var("DB_TYPE").unwrap_or("sqlite".to_string());
    if db_type == *"sqlite" {
        get_data_all_activity_by_server_sqlite(server_id).await
    } else if db_type == *"postgresql" {
        get_data_all_activity_by_server_postgresql(server_id).await
    } else {
        get_data_all_activity_by_server_sqlite(server_id).await
    }
}

pub async fn get_all_user_approximated_color() -> Result<Vec<UserColor>, AppError> {
    let db_type = env::var("DB_TYPE").unwrap_or("sqlite".to_string());
    if db_type == *"sqlite" {
        get_all_user_approximated_color_sqlite().await
    } else if db_type == *"postgresql" {
        get_all_user_approximated_color_postgres().await
    } else {
        get_all_user_approximated_color_sqlite().await
    }
}

pub async fn set_server_image(
    server_id: &String,
    image_type: &String,
    image: &String,
    image_url: &String,
) -> Result<(), AppError> {
    let db_type = env::var("DB_TYPE").unwrap_or("sqlite".to_string());
    if db_type == *"sqlite" {
        set_server_image_sqlite(server_id, image_type, image, image_url).await
    } else if db_type == *"postgresql" {
        set_server_image_postgresql(server_id, image_type, image, image_url).await
    } else {
        set_server_image_sqlite(server_id, image_type, image, image_url).await
    }
}

pub async fn get_server_image(
    server_id: &String,
    image_type: &String,
) -> Result<(Option<String>, Option<String>), AppError> {
    let db_type = env::var("DB_TYPE").unwrap_or("sqlite".to_string());
    if db_type == *"sqlite" {
        get_server_image_sqlite(server_id, image_type).await
    } else if db_type == *"postgresql" {
        get_server_image_postgresql(server_id, image_type).await
    } else {
        get_server_image_sqlite(server_id, image_type).await
    }
}
