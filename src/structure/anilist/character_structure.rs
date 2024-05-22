#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Name {
	#[serde(rename = "first")]
	pub first: Option<String>,

	#[serde(rename = "last")]
	pub last: Option<String>,

	#[serde(rename = "full")]
	pub full: Option<String>,

	#[serde(rename = "native")]
	pub native: Option<String>,

	#[serde(rename = "alternative")]
	pub alternative: Option<Vec<String>>,

	#[serde(rename = "alternativeSpoiler")]
	pub alternative_spoiler: Option<Vec<String>>,

	#[serde(rename = "userPreferred")]
	pub user_preferred: Option<String>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Image {
	#[serde(rename = "large")]
	pub large: Option<String>,

	#[serde(rename = "medium")]
	pub medium: Option<String>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DateOfBirth {
	#[serde(rename = "year")]
	pub year: Option<i32>,

	#[serde(rename = "month")]
	pub month: Option<i32>,

	#[serde(rename = "day")]
	pub day: Option<i32>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Character {
	#[serde(rename = "id")]
	pub id: Option<i32>,

	#[serde(rename = "name")]
	pub name: Option<Name>,

	#[serde(rename = "image")]
	pub image: Option<Image>,

	#[serde(rename = "description")]
	pub description: Option<String>,

	#[serde(rename = "gender")]
	pub gender: Option<String>,

	#[serde(rename = "dateOfBirth")]
	pub date_of_birth: Option<DateOfBirth>,

	#[serde(rename = "age")]
	pub age: Option<String>,

	#[serde(rename = "siteUrl")]
	pub site_url: Option<String>,

	#[serde(rename = "favourites")]
	pub favourites: Option<i32>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CharacterWrapper {
	#[serde(rename = "Character")]
	pub character: Option<Character>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CharacterDataWrapper {
	#[serde(rename = "data")]
	pub data: Option<CharacterWrapper>,
}
use serde::{Serialize, Deserialize};
