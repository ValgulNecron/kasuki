use serde::Deserialize;
use serde_json::json;
use serenity::all::{
    CommandInteraction, Context, CreateEmbed, CreateInteractionResponse,
    CreateInteractionResponseMessage, Timestamp,
};
use tracing::log::trace;

use crate::helper::convert_flavored_markdown::convert_anilist_flavored_to_discord_flavored_markdown;
use crate::helper::make_anilist_cached_request::make_request_anilist;
use crate::helper::trimer::trim;
use crate::constant::COLOR;
use crate::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use crate::structure::message::anilist_user::character::load_localization_character;

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

/// `CharacterWrapper` is an implementation block for the `CharacterWrapper` struct.
impl CharacterWrapper {
    /// `new_character_by_id` is an asynchronous function that creates a new character by ID.
    /// It takes an `id` as a parameter.
    /// `id` is an integer that represents the ID of the character.
    /// It returns a `Result` that contains a `CharacterWrapper` or an `AppError`.
    ///
    /// This function first defines a GraphQL query string that takes an `id` as a variable.
    /// It then creates a JSON object with the query string and the variable.
    /// The `id` variable is set to the `id` parameter.
    /// It makes a request to AniList with the JSON object and waits for the response.
    /// It then deserializes the response into a `CharacterWrapper` and returns it.
    ///
    /// # Arguments
    ///
    /// * `id` - An integer that represents the ID of the character.
    ///
    /// # Returns
    ///
    /// * `Result<CharacterWrapper, AppError>` - A Result that contains a `CharacterWrapper` or an `AppError`.
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

    /// `new_character_by_search` is an asynchronous function that creates a new character by search.
    /// It takes a `search` as a parameter.
    /// `search` is a reference to a String that represents the search query.
    /// It returns a `Result` that contains a `CharacterWrapper` or an `AppError`.
    ///
    /// This function first defines a GraphQL query string that takes a `search` as a variable.
    /// It then creates a JSON object with the query string and the variable.
    /// The `search` variable is set to the `search` parameter.
    /// It makes a request to AniList with the JSON object and waits for the response.
    /// It then deserializes the response into a `CharacterWrapper` and returns it.
    ///
    /// # Arguments
    ///
    /// * `search` - A reference to a String that represents the search query.
    ///
    /// # Returns
    ///
    /// * `Result<CharacterWrapper, AppError>` - A Result that contains a `CharacterWrapper` or an `AppError`.
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

/// `send_embed` is an asynchronous function that sends an embed message.
/// It takes `ctx`, `command_interaction`, and `data` as parameters.
/// `ctx` is a Context that represents the context.
/// `command_interaction` is a CommandInteraction that represents the command interaction.
/// `data` is a CharacterWrapper that represents the character wrapper.
///
/// This function first gets the guild ID from the `command_interaction`.
/// It then clones the character from the `data`.
/// It loads the localized character using the guild ID.
/// It clones the date of birth data from the character.
/// It creates a date of birth string from the date of birth data.
/// It replaces the placeholders in the description of the localized character with the actual data from the character.
/// It converts the AniList flavored markdown in the description to Discord flavored markdown.
/// It trims the description if it exceeds the limit.
/// It gets the native name and the user preferred name from the character and formats them into a character name.
/// It creates a new embed with the description, the thumbnail, the title, and the URL of the character.
/// It creates a new interaction response message with the embed.
/// It creates a new interaction response with the interaction response message.
/// It then sends the interaction response using the `command_interaction`.
///
/// # Arguments
///
/// * `ctx` - A Context that represents the context.
/// * `command_interaction` - A CommandInteraction that represents the command interaction.
/// * `data` - A CharacterWrapper that represents the character wrapper.
///
/// # Returns
///
/// * `Result<(), AppError>` - A Result that represents the result of the function. It returns an empty Ok if the function is successful, otherwise it returns an Err with an AppError.
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
