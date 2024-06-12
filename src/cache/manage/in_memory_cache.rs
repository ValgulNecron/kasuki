use crate::constant::CACHE;

pub async unsafe fn set_cache(input_json: String, response: String) {
    CACHE.insert(input_json, response).await;
}

pub async unsafe fn get_cache(input_json: String) -> Option<String> {
    CACHE.get(&input_json).await
}
