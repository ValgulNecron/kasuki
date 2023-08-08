use serde::Deserialize;
use serde_json::json;

use crate::cmd::general_module::html_parser::convert_to_discord_markdown;
use crate::cmd::general_module::lang_struct::CharacterLocalisedText;
use crate::cmd::general_module::request::make_request_anilist;
use crate::cmd::general_module::trim::trim;

#[derive(Deserialize)]
pub struct CharacterWrapper {
    pub data: CharacterData,
}

#[derive(Deserialize)]
pub struct CharacterData {
    #[serde(rename = "Character")]
    pub character: Character,
}

#[derive(Deserialize)]
pub struct Character {
    pub id: u32,
    pub name: Name,
    #[serde(rename = "siteUrl")]
    pub site_url: String,
    pub description: String,
    pub gender: String,
    pub age: String,
    #[serde(rename = "dateOfBirth")]
    pub date_of_birth: DateOfBirth,
    pub image: Image,
    pub favourites: u32,
    #[serde(rename = "modNotes")]
    pub mod_notes: Option<String>,
}

#[derive(Deserialize)]
pub struct Name {
    pub full: String,
    pub native: String,
    #[serde(rename = "userPreferred")]
    pub user_preferred: String,
}

#[derive(Deserialize)]
pub struct DateOfBirth {
    pub year: Option<u32>,
    pub month: Option<u32>,
    pub day: Option<u32>,
}

#[derive(Deserialize)]
pub struct Image {
    pub large: String,
}

impl CharacterWrapper {
    pub async fn new_character_by_id(
        value: i32,
        localised_text: CharacterLocalisedText,
    ) -> Result<CharacterWrapper, String> {
        let query_id: &str = "
        query ($name: Int) {
            Character(id: $name) {
            id
            name {
              full
              native
              userPreferred
            }
            siteUrl
            description
            gender
            age
            dateOfBirth {
              year
              month
              day
            }
            image {
              large
            }
            favourites
            modNotes
          }
        }
        ";
        let json = json!({"query": query_id, "variables": {"name": value}});
        let resp = make_request_anilist(json, false).await;
        let data: CharacterWrapper = match serde_json::from_str(&resp) {
            Ok(result) => result,
            Err(e) => {
                println!("Failed to parse JSON: {}", e);
                return Err(localised_text.error_no_character);
            }
        };
        return Ok(data);
    }

    pub async fn new_character_by_search(
        value: &String,
        localised_text: CharacterLocalisedText,
    ) -> Result<CharacterWrapper, String> {
        let query_string: &str = "
query ($name: String) {
	Character(search: $name) {
    id
    name {
      full
      native
      userPreferred
    }
    siteUrl
    description
    gender
    age
    dateOfBirth {
      year
      month
      day
    }
    image {
      large
    }
    favourites
    modNotes
  }
}
";
        let json = json!({"query": query_string, "variables": {"name": value}});
        let resp = make_request_anilist(json, false).await;
        let data: CharacterWrapper = match serde_json::from_str(&resp) {
            Ok(result) => result,
            Err(e) => {
                println!("Failed to parse JSON: {}", e);
                return Err(localised_text.error_no_character);
            }
        };
        return Ok(data);
    }

    pub fn get_name(&self) -> String {
        format!(
            "{}/{}",
            self.data.character.name.user_preferred, self.data.character.name.native
        )
    }

    pub fn get_desc(&self, localised_text: CharacterLocalisedText) -> String {
        let mut desc = self.data.character.description.clone();
        desc = format!("{} {}", localised_text.desc.clone(), desc);
        desc = convert_to_discord_markdown(desc);
        let lenght_diff = 4096 - desc.len() as i32;
        if lenght_diff <= 0 {
            desc = trim(desc, lenght_diff);
        }

        desc
    }

    pub fn get_info(&self, localised_text: CharacterLocalisedText) -> String {
        let age = &self.get_age();
        let gender = &self.get_gender();
        let favourite = &self.get_fav();
        let date_of_birth = &self.get_date_of_birth();
        let full_description = format!(
            "{}{}{}{}{}{}{}{}.",
            &localised_text.age,
            age,
            &localised_text.gender,
            gender,
            &localised_text.date_of_birth,
            date_of_birth,
            &localised_text.favourite,
            favourite,
        );
        full_description
    }

    pub fn get_age(&self) -> String {
        self.data.character.age.clone()
    }

    pub fn get_gender(&self) -> String {
        self.data.character.gender.clone()
    }

    pub fn get_fav(&self) -> u32 {
        self.data.character.favourites.clone()
    }

    pub fn get_date_of_birth(&self) -> String {
        format!(
            "{}/{}/{}",
            self.data
                .character
                .date_of_birth
                .month
                .clone()
                .unwrap_or_else(|| 0),
            self.data
                .character
                .date_of_birth
                .day
                .clone()
                .unwrap_or_else(|| 0),
            self.data
                .character
                .date_of_birth
                .year
                .clone()
                .unwrap_or_else(|| 0)
        )
    }

    pub fn get_image(&self) -> String {
        self.data.character.image.large.clone()
    }

    pub fn get_url(&self) -> String {
        self.data.character.site_url.clone()
    }
}
