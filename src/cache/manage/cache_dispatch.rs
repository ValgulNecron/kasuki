use crate::cache::manage::in_memory_cache;

pub async fn set_cache(input_json: String, response: String, cache_type: &str) {
    unsafe {
        if cache_type == "in-memory" {
            in_memory_cache::set_cache(input_json, response).await;
        } else {
            in_memory_cache::set_cache(input_json, response).await;
        }
    }
}

pub async fn get_cache(input_json: String, cache_type: &str) -> Option<String> {
    unsafe {
        if cache_type == "in-memory" {
            in_memory_cache::get_cache(input_json).await
        } else {
            in_memory_cache::get_cache(input_json).await
        }
    }
}
