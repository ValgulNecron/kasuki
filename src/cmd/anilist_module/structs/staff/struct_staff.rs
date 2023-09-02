use serde::Deserialize;
use serde_json::json;

use crate::cmd::general_module::function::html_parser::convert_to_discord_markdown;
use crate::cmd::general_module::function::request::make_request_anilist;
use crate::cmd::general_module::function::trim::trim;
use crate::cmd::lang_struct::embed::anilist::struct_lang_staff::StaffLocalisedText;

#[derive(Debug, Deserialize)]
pub struct Name {
    pub full: Option<String>,
    pub native: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Image {
    pub large: String,
}

#[derive(Debug, Deserialize)]
pub struct Date {
    pub year: Option<i32>,
    pub month: Option<i32>,
    pub day: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct Title {
    pub romaji: Option<String>,
    pub english: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Node {
    pub title: Title,
}

#[derive(Debug, Deserialize)]
pub struct StaffMedia {
    pub edges: Vec<Edge>,
}

#[derive(Debug, Deserialize)]
pub struct Edge {
    pub node: Node,
    #[serde(rename = "roleNotes")]
    pub role_notes: Option<String>,
    #[serde(rename = "relationType")]
    pub relation_type: Option<String>,
    #[serde(rename = "staffRole")]
    pub staff_role: String,
}

#[derive(Debug, Deserialize)]
pub struct Character {
    pub name: Name,
    pub image: Image,
}

#[derive(Debug, Deserialize)]
pub struct Characters {
    pub nodes: Vec<Character>,
}

#[derive(Debug, Deserialize)]
pub struct Staff {
    pub name: Name,
    pub id: i32,
    #[serde(rename = "languageV2")]
    pub language_v2: String,
    pub image: Image,
    pub description: String,
    #[serde(rename = "primaryOccupations")]
    pub primary_occupations: Vec<String>,
    pub gender: Option<String>,
    #[serde(rename = "dateOfBirth")]
    pub date_of_birth: Date,
    #[serde(rename = "dateOfDeath")]
    pub date_of_death: Date,
    pub age: Option<i32>,
    #[serde(rename = "yearsActive")]
    pub years_active: Vec<i32>,
    #[serde(rename = "homeTown")]
    pub home_town: Option<String>,
    #[serde(rename = "siteUrl")]
    pub site_url: String,
    #[serde(rename = "staffMedia")]
    pub staff_media: StaffMedia,
    pub characters: Characters,
}

#[derive(Debug, Deserialize)]
pub struct StaffData {
    #[serde(rename = "Staff")]
    pub staff: Staff,
}

#[derive(Debug, Deserialize)]
pub struct StaffWrapper {
    pub data: StaffData,
}

impl StaffWrapper {
    pub async fn new_staff_by_id(id: i32) -> Result<StaffWrapper, String> {
        let query_id: &str = "
query ($name: Int, $limit1: Int = 5, $limit2: Int = 15) {
	Staff(id: $name){
    name {
      full
      native
    }
    id
    languageV2
    image {
      large
    }
    description
    primaryOccupations
    gender
    dateOfBirth {
      year
      month
      day
    }
    dateOfDeath {
      year
      month
      day
    }
    age
    yearsActive
    homeTown
    siteUrl
    staffMedia(perPage: $limit1){
      edges{
        node {
          title {
            romaji
            english
          }
        }
        roleNotes
        relationType
        staffRole
      }
    }
    characters(perPage: $limit2) {
      nodes {
        name {
          full
        }
        image {
          large
        }
      }
    }
  }
}
";
        let json = json!({"query": query_id, "variables": {"name": id}});
        let resp = make_request_anilist(json, false).await;
        let data: StaffWrapper = match serde_json::from_str(&resp) {
            Ok(result) => result,
            Err(e) => {
                println!("Failed to parse JSON: {}", e);
                return Err(String::from("Error: Failed to retrieve user data"));
            }
        };
        return Ok(data);
    }

    pub async fn new_staff_by_search(search: &String) -> Result<StaffWrapper, String> {
        let query_string: &str = "
query ($name: String, $limit1: Int = 5, $limit2: Int = 15) {
	Staff(search: $name){
    name {
      full
      native
    }
    id
    languageV2
    image {
      large
    }
    description
    primaryOccupations
    gender
    dateOfBirth {
      year
      month
      day
    }
    dateOfDeath {
      year
      month
      day
    }
    age
    yearsActive
    homeTown
    siteUrl
    staffMedia(perPage: $limit1){
      edges{
        node {
          title {
            romaji
            english
          }
        }
        roleNotes
        relationType
        staffRole
      }
    }
    characters(perPage: $limit2) {
      nodes {
        name {
          full
        }
        image {
          large
        }
      }
    }
  }
}
";
        let json = json!({"query": query_string, "variables": {"name": search}});
        let resp = make_request_anilist(json, false).await;
        let data: StaffWrapper = match serde_json::from_str(&resp) {
            Ok(result) => result,
            Err(e) => {
                println!("Failed to parse JSON: {}", e);
                return Err(String::from("Error: Failed to retrieve user data"));
            }
        };
        return Ok(data);
    }

    pub fn format_va(&self) -> String {
        let formatted_nodes_va: Vec<String> = self
            .data
            .staff
            .characters
            .nodes
            .iter()
            .map(|character| {
                let name_natif = character
                    .name
                    .native
                    .clone()
                    .unwrap_or_else(|| "N/A".to_string());
                let name_full = character
                    .name
                    .full
                    .clone()
                    .unwrap_or_else(|| "N/A".to_string());
                let name = format!("{} / {}", name_natif, name_full);
                format!("{}", name)
            })
            .collect();

        formatted_nodes_va.join(",\n")
    }

    pub fn format_role(&self) -> String {
        let formatted_edges_role: Vec<String> = self
            .data
            .staff
            .staff_media
            .edges
            .iter()
            .map(|edge| {
                let title = match &edge.node.title.romaji {
                    Some(romaji) => romaji.clone(),
                    None => match &edge.node.title.english {
                        Some(english) => english.clone(),
                        None => "".to_string(),
                    },
                };
                let staff_role = &edge.staff_role;
                format!("{} ({})", title, staff_role)
            })
            .collect();
        formatted_edges_role.join("\n")
    }

    pub fn get_url(&self) -> String {
        format!("https://anilist.co/staff/{}", &self.data.staff.id)
    }

    pub fn get_name(&self) -> String {
        format!(
            "{}/{}",
            self.data
                .staff
                .name
                .native
                .clone()
                .unwrap_or_else(|| "N/A".to_string()),
            self.data
                .staff
                .name
                .full
                .clone()
                .unwrap_or_else(|| "N/A".to_string())
        )
    }

    pub fn get_desc(&self, localised_text: &StaffLocalisedText) -> String {
        let lang = self.get_lang();
        let hometown = self.get_hometown();
        let occupations_string = self.get_occupation();
        let birth = self.get_birth();
        let death = self.get_death();

        let mut desc = self.data.staff.description.clone();
        desc = convert_to_discord_markdown(desc);
        let mut full_description = format!(
            "{}{}{}{}{}{}{}{}{}{}",
            &localised_text.date_of_birth,
            birth,
            &localised_text.date_of_death,
            death,
            &localised_text.hometown,
            hometown,
            &localised_text.primary_language,
            lang,
            &localised_text.primary_occupation,
            occupations_string
        );
        let lenght_diff = 4096 - full_description.len() as i32;
        if lenght_diff <= 0 {
            desc = trim(desc, lenght_diff);

            full_description = format!(
                "{}{}{}{}{}{}{}{}{}{}\n\n{}",
                &localised_text.date_of_birth,
                birth,
                &localised_text.date_of_death,
                death,
                &localised_text.hometown,
                hometown,
                &localised_text.primary_language,
                lang,
                &localised_text.primary_occupation,
                occupations_string,
                desc
            );
        }

        full_description
    }

    pub fn get_birth(&self) -> String {
        format!(
            "{}/{}/{}",
            &self.data.staff.date_of_birth.month.unwrap_or_else(|| 0),
            &self.data.staff.date_of_birth.day.unwrap_or_else(|| 0),
            &self.data.staff.date_of_birth.year.unwrap_or_else(|| 0)
        )
    }

    pub fn get_death(&self) -> String {
        format!(
            "{}/{}/{}",
            &self.data.staff.date_of_death.month.unwrap_or_else(|| 0),
            &self.data.staff.date_of_death.day.unwrap_or_else(|| 0),
            &self.data.staff.date_of_death.year.unwrap_or_else(|| 0)
        )
    }

    pub fn get_lang(&self) -> String {
        self.data.staff.language_v2.clone()
    }

    pub fn get_image(&self) -> String {
        self.data.staff.image.large.clone()
    }

    pub fn get_hometown(&self) -> String {
        self.data
            .staff
            .home_town
            .clone()
            .unwrap_or_else(|| "N/A".to_string())
    }

    pub fn get_occupation(&self) -> String {
        let max_limit = 5;
        let limited_occupations: Vec<String> = self
            .data
            .staff
            .primary_occupations
            .iter()
            .take(max_limit)
            .cloned()
            .collect();
        limited_occupations.join(", ")
    }
}
