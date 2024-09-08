use crate::structure::run::anilist::seiyuu_id;

#[cynic::schema("anilist")]

mod schema {}

#[derive(cynic::QueryVariables, Debug, Clone)]

pub struct SeiyuuSearchVariables<'a> {
    pub per_page: Option<i32>,
    pub search: Option<&'a str>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Query", variables = "SeiyuuSearchVariables")]

pub struct SeiyuuSearch {
    #[cynic(rename = "Page")]
    pub page: Option<Page>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(variables = "SeiyuuSearchVariables")]

pub struct Page {
    #[arguments(search: $ search)]
    pub staff: Option<Vec<Option<Staff>>>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(variables = "SeiyuuSearchVariables")]

pub struct Staff {
    pub site_url: Option<String>,
    pub image: Option<StaffImage>,
    pub name: Option<StaffName>,
    #[arguments(perPage: $ per_page, sort: "FAVOURITES")]
    pub characters: Option<CharacterConnection>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]

pub struct StaffName {
    pub user_preferred: Option<String>,
    pub native: Option<String>,
    pub full: Option<String>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]

pub struct StaffImage {
    pub large: Option<String>,
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
    pub full: Option<String>,
    pub native: Option<String>,
    pub user_preferred: Option<String>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]

pub struct CharacterImage {
    pub large: Option<String>,
}

#[derive(cynic::Enum, Clone, Copy, Debug)]

pub enum CharacterSort {
    Id,
    IdDesc,
    Role,
    RoleDesc,
    SearchMatch,
    Favourites,
    FavouritesDesc,
    Relevance,
}

impl From<CharacterImage> for seiyuu_id::CharacterImage {
    fn from(character_image: CharacterImage) -> Self {

        Self {
            large: character_image.large,
        }
    }
}

impl From<CharacterName> for seiyuu_id::CharacterName {
    fn from(character_name: CharacterName) -> Self {

        Self {
            full: character_name.full,
            native: character_name.native,
            user_preferred: character_name.user_preferred,
        }
    }
}

impl From<Character> for seiyuu_id::Character {
    fn from(character: Character) -> Self {

        Self {
            image: character.image.map(|image| image.into()),
            name: character.name.map(|name| name.into()),
        }
    }
}

impl From<CharacterConnection> for seiyuu_id::CharacterConnection {
    fn from(character_connection: CharacterConnection) -> Self {

        let nodes: Option<Vec<Option<seiyuu_id::Character>>> =
            character_connection.nodes.map(|nodes| {

                nodes
                    .into_iter()
                    .filter_map(|node| {

                        node.map(|node| {

                            let node: seiyuu_id::Character = node.into();

                            Some(node)
                        })
                    })
                    .collect()
            });

        Self { nodes }
    }
}

impl From<StaffImage> for seiyuu_id::StaffImage {
    fn from(staff_image: StaffImage) -> Self {

        Self {
            large: staff_image.large,
        }
    }
}

impl From<StaffName> for seiyuu_id::StaffName {
    fn from(staff_name: StaffName) -> Self {

        Self {
            full: staff_name.full,
            native: staff_name.native,
            user_preferred: staff_name.user_preferred,
        }
    }
}

impl From<Staff> for seiyuu_id::Staff {
    fn from(staff: Staff) -> Self {

        Self {
            site_url: staff.site_url,
            image: staff.image.map(|image| image.into()),
            name: staff.name.map(|name| name.into()),
            characters: staff.characters.map(|characters| characters.into()),
        }
    }
}
