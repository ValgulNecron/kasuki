#[cynic::schema("anilist")]
mod schema {}

#[derive(cynic::QueryVariables, Debug, Clone)]
pub struct StaffQuerryVariables<'a> {
    pub id: Option<i32>,
    pub search: Option<&'a str>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Query", variables = "StaffQuerryVariables")]
pub struct StaffQuerry {
    #[arguments(id: $ id, search: $ search)]
    #[cynic(rename = "Staff")]
    pub staff: Option<Staff>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Staff")]
pub struct Staff {
    pub id: i32,
    pub language_v2: Option<String>,
    pub name: Option<StaffName>,
    pub image: Option<StaffImage>,
    pub primary_occupations: Option<Vec<Option<String>>>,
    pub gender: Option<String>,
    pub date_of_birth: Option<FuzzyDate>,
    pub description: Option<String>,
    pub date_of_death: Option<FuzzyDate>,
    pub age: Option<i32>,
    pub years_active: Option<Vec<Option<i32>>>,
    pub home_town: Option<String>,
    pub site_url: Option<String>,
    pub staff_media: Option<MediaConnection>,
    pub characters: Option<CharacterConnection>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct StaffImage {
    pub large: Option<String>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct StaffName {
    pub user_preferred: Option<String>,
    pub native: Option<String>,
    pub full: Option<String>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct MediaConnection {
    pub edges: Option<Vec<Option<MediaEdge>>>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct MediaEdge {
    pub role_notes: Option<String>,
    pub staff_role: Option<String>,
    pub relation_type: Option<MediaRelation>,
    pub node: Option<Media>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct Media {
    pub title: Option<MediaTitle>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct MediaTitle {
    pub user_preferred: Option<String>,
    pub native: Option<String>,
    pub romaji: Option<String>,
    pub english: Option<String>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct FuzzyDate {
    pub year: Option<i32>,
    pub month: Option<i32>,
    pub day: Option<i32>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct CharacterConnection {
    pub nodes: Option<Vec<Option<Character>>>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct Character {
    pub image: Option<CharacterImage>,
    pub name: Option<CharacterName>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct CharacterName {
    pub user_preferred: Option<String>,
    pub native: Option<String>,
    pub full: Option<String>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct CharacterImage {
    pub large: Option<String>,
}

#[derive(cynic::Enum, Clone, Copy, Debug)]
pub enum MediaRelation {
    Adaptation,
    Prequel,
    Sequel,
    Parent,
    SideStory,
    Character,
    Summary,
    Alternative,
    SpinOff,
    Other,
    Source,
    Compilation,
    Contains,
}
