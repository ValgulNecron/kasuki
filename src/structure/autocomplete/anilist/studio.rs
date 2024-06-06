#[cynic::schema("anilist")]
mod schema {}

#[derive(cynic::QueryVariables, Debug)]
pub struct StudioAutocompleteVariables<'a> {
    pub search: Option<&'a str>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "StudioAutocompleteVariables")]
pub struct StudioAutocomplete {
    #[arguments(perPage: 25)]
    #[cynic(rename = "Page")]
    pub page: Option<Page>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(variables = "StudioAutocompleteVariables")]
pub struct Page {
    #[arguments(search: $ search)]
    pub studios: Option<Vec<Option<Studio>>>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct Studio {
    pub name: String,
    pub id: i32,
}
