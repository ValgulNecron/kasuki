#[cynic::schema("anilist")]

mod schema {}

#[derive(cynic::QueryVariables, Debug, Clone)]

pub struct SeiyuuIdVariables {
	pub id: Option<i32>,
	pub per_page: Option<i32>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Query", variables = "SeiyuuIdVariables")]

pub struct SeiyuuId {
	#[cynic(rename = "Page")]
	pub page: Option<Page>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(variables = "SeiyuuIdVariables")]

pub struct Page {
	#[arguments(id: $ id)]
	pub staff: Option<Vec<Option<Staff>>>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(variables = "SeiyuuIdVariables")]

pub struct Staff {
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
