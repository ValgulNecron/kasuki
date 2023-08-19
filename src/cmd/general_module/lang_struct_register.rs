use serde::{Deserialize, Serialize};
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AvailableLang {
    pub lang: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LangRegister {
    pub code: String,
    pub name: String,
    pub description: String,
    pub option1: String,
    pub option1_desc: String,
}
