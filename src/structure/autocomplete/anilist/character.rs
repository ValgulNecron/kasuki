#[cynic::schema("anilist")]
mod schema {}

#[derive(cynic::QueryVariables, Debug, Clone)]
pub struct CharacterAutocompleteVariables<'a> {
    pub search: Option<&'a str>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Query", variables = "CharacterAutocompleteVariables")]
pub struct CharacterAutocomplete {
    #[arguments(perPage: 25)]
    #[cynic(rename = "Page")]
    pub page: Option<Page>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(variables = "CharacterAutocompleteVariables")]
pub struct Page {
    #[arguments(search: $ search)]
    pub characters: Option<Vec<Option<Character>>>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct Character {
    pub id: i32,
    pub name: Option<CharacterName>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct CharacterName {
    pub full: Option<String>,
    pub user_preferred: Option<String>,
    pub native: Option<String>,
}
