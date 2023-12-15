use crate::common::html_parser::convert_to_discord_markdown;
use crate::common::make_anilist_request::make_request_anilist;
use crate::common::trimer::trim;
use crate::constant::{COLOR, COMMAND_SENDING_ERROR};
use crate::error_enum::AppError;
use crate::error_enum::AppError::MediaGettingError;
use crate::lang_struct::anilist::character::load_localization_character;
use serde::Deserialize;
use serde_json::json;
use serenity::all::{
    CommandInteraction, Context, CreateEmbed, CreateInteractionResponse,
    CreateInteractionResponseMessage, Timestamp,
};

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
    pub async fn new_character_by_id(value: i32) -> Result<CharacterWrapper, AppError> {
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
        serde_json::from_str(&resp)
            .map_err(|_| MediaGettingError(String::from("Error getting this media.")))
    }

    pub async fn new_character_by_search(value: &String) -> Result<CharacterWrapper, AppError> {
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
        serde_json::from_str(&resp)
            .map_err(|_| MediaGettingError(String::from("Error getting this media.")))
    }
}

pub async fn send_embed(
    ctx: &Context,
    command: &CommandInteraction,
    data: CharacterWrapper,
) -> Result<(), AppError> {
    let guild_id = match command.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };

    let character = data.data.character.clone();

    let character_localised = load_localization_character(guild_id).await?;

    let dob_data = character.date_of_birth.clone();
    let mut dob_string = String::new();

    let mut mo: bool = false;
    let mut da: bool = false;

    match dob_data.month {
        Some(m) => {
            dob_string.push_str(format!("{:02}", m).as_str());
            mo = true
        }
        None => {}
    }

    match dob_data.day {
        Some(d) => {
            if mo {
                dob_string.push_str("/")
            }
            dob_string.push_str(format!("{:02}", d).as_str());
            da = true
        }
        None => {}
    }

    match dob_data.year {
        Some(y) => {
            if da {
                dob_string.push_str("/")
            }
            dob_string.push_str(format!("{:04}", y).as_str());
        }
        None => {}
    }

    let mut dob = String::new();
    if dob_string != String::new() {
        dob = character_localised
            .date_of_birth
            .replace("$date$", dob_string.as_str())
    }

    let mut desc = character_localised
        .desc
        .replace("$age$", character.age.as_str())
        .replace("$gender$", character.gender.as_str())
        .replace("$date_of_birth$", dob.as_str())
        .replace("$fav$", character.favourites.to_string().as_str())
        .replace("$desc$", character.description.as_str());

    desc = convert_to_discord_markdown(desc);
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

    command
        .create_response(&ctx.http, builder)
        .await
        .map_err(|_| COMMAND_SENDING_ERROR.clone())
}