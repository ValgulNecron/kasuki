use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct AutocompleteOption {
    pub name: String,
    pub value: String,
}
