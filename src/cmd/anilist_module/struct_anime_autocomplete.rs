use serde::Deserialize;
use crate::cmd::anilist_module::struct_autocomplete_media::AutocompleteMedia;

#[derive(Debug, Deserialize)]
pub struct AnimePage {
    pub media: Option<Vec<Option<AutocompleteMedia>>>,
}

#[derive(Debug, Deserialize)]
pub struct AnimePageData {
    #[serde(rename = "Page")]
    pub page: AnimePage,
}

#[derive(Debug, Deserialize)]
pub struct AnimePageWrapper {
    pub data: AnimePageData,
}
