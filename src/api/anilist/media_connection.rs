use serde::{Deserialize, Serialize};

use crate::api::anilist::media::MediaAPISort;

#[derive(Serialize, Debug, Deserialize, Clone)]
pub struct MediaConnectionAPIBuilder {
    sort: Option<MediaAPISort>,
    #[serde(rename = "type")]
    media_type: Option<String>,
    page: Option<u32>,
    per_page: Option<u32>,
    query: Option<String>,
    media: Option<bool>,
    media_edges: Option<bool>,
}

const QUERY: &str = r#"

"#;

impl MediaConnectionAPIBuilder {
    pub fn new() -> Self {
        Self {
            sort: None,
            media_type: None,
            page: None,
            per_page: None,
            query: None,
            media: None,
            media_edges: None,
        }
    }

    pub fn sort(mut self, sort: MediaAPISort) -> Self {
        self.sort = Some(sort);
        self
    }

    pub fn media_type(mut self, media_type: String) -> Self {
        self.media_type = Some(media_type);
        self
    }

    pub fn page(mut self, page: u32) -> Self {
        self.page = Some(page);
        self
    }

    pub fn per_page(mut self, per_page: u32) -> Self {
        self.per_page = Some(per_page);
        self
    }

    pub fn build(self, limit: Option<u32>, actual: Option<u32>) -> Self {
        self
    }

    pub fn get_query(self) -> Option<String> {
        self.query
    }
}
