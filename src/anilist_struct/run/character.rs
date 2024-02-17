use serde::Deserialize;
use serde_json::json;
use serenity::all::{
    CommandInteraction, Context, CreateEmbed, CreateInteractionResponse,
    CreateInteractionResponseMessage, Timestamp,
};
use tracing::log::trace;

use crate::common::anilist_to_discord_markdown::convert_anilist_flavored_to_discord_flavored_markdown;
use crate::common::make_anilist_request::make_request_anilist;
use crate::common::trimer::trim;
use crate::constant::COLOR;
use crate::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use crate::lang_struct::anilist::character::load_localization_character;

#[derive(Debug, Deserialize, Clone)]
pub struct CharacterWrapper {
    pub data: CharacterData,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CharacterData {
    #[serde(rename = "Character")]
    pub character: Character,
}

#[derive(Debug, Deserialize, Clone)]
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

#[derive(Debug, Deserialize, Clone)]
pub struct Name {
    pub full: String,
    pub native: String,
    #[serde(rename = "userPreferred")]
    pub user_preferred: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DateOfBirth {
    pub year: Option<u32>,
    pub month: Option<u32>,
    pub day: Option<u32>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Image {
    pub large: String,
}

impl CharacterWrapper {
    pub async fn new_character_by_id(id: i32) -> Result<CharacterWrapper, AppError> {
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
        let json = json!({"query": query_id, "variables": {"name": id}});
        trace!("{:#?}", json);
        let resp = make_request_anilist(json, false).await;
        trace!("{:#?}", resp);
        serde_json::from_str(&resp).map_err(|e| AppError {
            message: format!("Error getting the character with id {}. {}", id, e),
            error_type: ErrorType::WebRequest,
            error_response_type: ErrorResponseType::Message,
        })
    }

    pub async fn new_character_by_search(search: &String) -> Result<CharacterWrapper, AppError> {
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
        let json = json!({"query": query_string, "variables": {"name": search}});
        trace!("{:#?}", json);
        let resp = make_request_anilist(json, false).await;
        trace!("{:#?}", resp);
        serde_json::from_str(&resp).map_err(|e| AppError {
            message: format!("Error getting the character with name {}. {}", search, e),
            error_type: ErrorType::WebRequest,
            error_response_type: ErrorResponseType::Message,
        })
    }
}

pub async fn send_embed(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    data: CharacterWrapper,
) -> Result<(), AppError> {
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };

    trace!("{:#?}", guild_id);

    let character = data.data.character.clone();

    let character_localised = load_localization_character(guild_id).await?;

    let date_of_birth_data = character.date_of_birth.clone();
    let mut date_of_birth_string = String::new();

    let mut has_month: bool = false;
    let mut has_day: bool = false;

    if let Some(m) = date_of_birth_data.month {
        date_of_birth_string.push_str(format!("{:02}", m).as_str());
        has_month = true
    }

    if let Some(d) = date_of_birth_data.day {
        if has_month {
            date_of_birth_string.push('/')
        }
        date_of_birth_string.push_str(format!("{:02}", d).as_str());
        has_day = true
    }

    if let Some(y) = date_of_birth_data.year {
        if has_day {
            date_of_birth_string.push('/')
        }
        date_of_birth_string.push_str(format!("{:04}", y).as_str());
    }

    let mut date_of_birth = String::new();
    if date_of_birth_string != String::new() {
        date_of_birth = character_localised
            .date_of_birth
            .replace("$date$", date_of_birth_string.as_str())
    }

    let mut desc = character_localised
        .desc
        .replace("$age$", character.age.as_str())
        .replace("$gender$", character.gender.as_str())
        .replace("$date_of_birth$", date_of_birth.as_str())
        .replace("$fav$", character.favourites.to_string().as_str())
        .replace("$desc$", character.description.as_str());

    desc = convert_anilist_flavored_to_discord_flavored_markdown(desc);
    let lenght_diff = 4096 - desc.len() as i32;
    if lenght_diff <= 0 {
        desc = trim(desc, lenght_diff)
    }

    let native = character.name.native;
    let user_pref = character.name.user_preferred;
    let character_name = format!("{}/{}", user_pref, native);

    let builder_embed = CreateEmbed::new()
        .timestamp(Timestamp::now())
        .color(COLOR)
        .description(desc)
        .thumbnail(character.image.large)
        .title(character_name)
        .url(character.site_url);

    let builder_message = CreateInteractionResponseMessage::new().embed(builder_embed);

    let builder = CreateInteractionResponse::Message(builder_message);

    command_interaction
        .create_response(&ctx.http, builder)
        .await
        .map_err(|e| AppError {
            message: format!("Error sending the character embed. {}", e),
            error_type: ErrorType::Command,
            error_response_type: ErrorResponseType::Message,
        })?;
    Ok(())
}
