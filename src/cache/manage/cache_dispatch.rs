use crate::cache::manage::in_memory_cache;
use crate::constant::CONFIG;

pub async fn set_cache(input_json: String, response: String) {
    let cache_type = unsafe { CONFIG.bot.config.cache_type.clone() };
    unsafe {
        if cache_type == "in-memory" {
            in_memory_cache::set_cache(input_json, response).await;
        } else {
            in_memory_cache::set_cache(input_json, response).await;
        }
    }
}

pub async fn get_cache(input_json: String) -> Option<String> {
    let cache_type = unsafe { CONFIG.bot.config.cache_type.clone() };
    unsafe {
        if cache_type == "in-memory" {
            in_memory_cache::get_cache(input_json).await
        } else {
            in_memory_cache::get_cache(input_json).await
        }
    }
}
