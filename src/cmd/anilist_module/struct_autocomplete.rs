use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct AutocompleteAnimeOption {
    pub name: String,
    pub value: String,
}
