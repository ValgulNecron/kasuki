use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ErrorLocalisedText {
    pub error_title: String,
    pub module_off: String,
    pub forgot_module: String,
    pub no_token: String,
    pub no_base_url: String,
    pub not_implemented: String,
    pub error_request: String,
    pub error_no_avatar: String,
    pub error_parsing_json: String,
    pub error_url: String,
    pub error_resolving_value: String,
    pub admin_instance_error: String,
    pub error_option: String,
    pub error_creating_header: String,
    pub error_getting_response_from_url: String,
    pub error_getting_bytes: String,
    pub error_writing_file: String,
    pub error_file_type: String,
    pub error_file_extension: String,
    pub error_no_anime_specified: String,
    pub error_not_nsfw: String,
}
