#[cynic::schema("anilist")]
mod schema {}

#[derive(cynic::QueryVariables, Debug, Clone)]
pub struct UserAutocompleteVariables<'a> {
    pub search: Option<&'a str>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Query", variables = "UserAutocompleteVariables")]
pub struct UserAutocomplete {
    #[arguments(perPage: 25)]
    #[cynic(rename = "Page")]
    pub page: Option<Page>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(variables = "UserAutocompleteVariables")]
pub struct Page {
    #[arguments(search: $ search)]
    pub users: Option<Vec<Option<User>>>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct User {
    pub id: i32,
    pub name: String,
}
