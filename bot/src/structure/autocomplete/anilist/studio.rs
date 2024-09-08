#[cynic::schema("anilist")]

mod schema {}

#[derive(cynic::QueryVariables, Debug, Clone)]

pub struct StudioAutocompleteVariables<'a> {
    pub search: Option<&'a str>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Query", variables = "StudioAutocompleteVariables")]

pub struct StudioAutocomplete {
    #[arguments(perPage: 25)]
    #[cynic(rename = "Page")]
    pub page: Option<Page>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(variables = "StudioAutocompleteVariables")]

pub struct Page {
    #[arguments(search: $ search)]
    pub studios: Option<Vec<Option<Studio>>>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]

pub struct Studio {
    pub name: String,
    pub id: i32,
}
