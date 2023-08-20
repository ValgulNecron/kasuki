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
}

