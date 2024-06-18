use crate::constant::ANILIST_CACHE;

pub async unsafe fn set_cache(input_json: String, response: String) {
    ANILIST_CACHE.insert(input_json, response).await;
}

pub async unsafe fn get_cache(input_json: String) -> Option<String> {
    ANILIST_CACHE.get(&input_json).await
}
