use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AutocompleteName {
    pub full: String,
    #[serde(rename = "userPreferred")]
    pub user_preferred: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AutocompleteStaff {
    pub id: u32,
    pub name: Option<AutocompleteName>,
}

#[derive(Debug, Deserialize)]
pub struct StaffPage {
    pub staff: Option<Vec<Option<AutocompleteStaff>>>,
}

#[derive(Debug, Deserialize)]
pub struct StaffPageData {
    #[serde(rename = "Page")]
    pub page: StaffPage,
}

#[derive(Debug, Deserialize)]
pub struct StaffPageWrapper {
    pub data: StaffPageData,
}
