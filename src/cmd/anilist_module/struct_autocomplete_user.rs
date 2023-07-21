use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AutocompleteUser {
    pub id: u32,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct UserPage {
    pub users: Option<Vec<Option<AutocompleteUser>>>,
}

#[derive(Debug, Deserialize)]
pub struct UserPageData {
    #[serde(rename = "Page")]
    pub page: UserPage,
}

#[derive(Debug, Deserialize)]
pub struct UserPageWrapper {
    pub data: UserPageData,
}
