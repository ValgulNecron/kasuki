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
	#[arguments(perPage: $ per_page, sort: "FAVOURITES")]
	pub characters: Option<CharacterConnection>,
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
}

#[derive(cynic::QueryFragment, Debug, Clone)]

pub struct CharacterImage {
	pub large: Option<String>,
}

impl From<CharacterImage> for seiyuu_id::CharacterImage {
	fn from(character_image: CharacterImage) -> Self {
		Self {
			large: character_image.large,
		}
	}
}

impl From<Character> for seiyuu_id::Character {
	fn from(character: Character) -> Self {
		Self {
			image: character.image.map(|image| image.into()),
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

impl From<Staff> for seiyuu_id::Staff {
	fn from(staff: Staff) -> Self {
		Self {
			image: staff.image.map(|image| image.into()),
			characters: staff.characters.map(|characters| characters.into()),
		}
	}
}
