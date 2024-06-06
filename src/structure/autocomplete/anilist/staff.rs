#[cynic::schema("anilist")]
mod schema {}

#[derive(cynic::QueryVariables, Debug)]
pub struct StaffAutocompleteVariables<'a> {
    pub search: Option<&'a str>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "StaffAutocompleteVariables")]
pub struct StaffAutocomplete {
    #[arguments(perPage: 25)]
    #[cynic(rename = "Page")]
    pub page: Option<Page>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(variables = "StaffAutocompleteVariables")]
pub struct Page {
    #[arguments(search: $ search)]
    pub staff: Option<Vec<Option<Staff>>>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct Staff {
    pub id: i32,
    pub name: Option<StaffName>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct StaffName {
    pub native: Option<String>,
    pub user_preferred: Option<String>,
    pub full: Option<String>,
}
