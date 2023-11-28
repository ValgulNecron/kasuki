use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Credit {
    desc: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Locale {
    title: String,
    credits: Vec<Credit>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Localization {
    en: Locale,
    fr: Locale,
    jp: Locale,
    de: Locale,
}
