use std::error::Error;

use serenity::all::{
    CommandInteraction, Context, CreateEmbed, CreateInteractionResponse,
    CreateInteractionResponseMessage, Timestamp,
};
use tracing::log::trace;

use crate::constant::COLOR;
use crate::helper::convert_flavored_markdown::convert_anilist_flavored_to_discord_flavored_markdown;
use crate::helper::error_management::error_enum::{ResponseError, UnknownResponseError};
use crate::helper::trimer::trim;
use crate::structure::message::anilist_user::character::load_localization_character;

#[cynic::schema("anilist")]
mod schema {}

#[derive(cynic::QueryVariables, Debug, Clone)]
pub struct CharacterQuerryIdVariables {
    pub id: Option<i32>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Query", variables = "CharacterQuerryIdVariables")]
pub struct CharacterQuerryId {
    #[arguments(id: $ id)]
    #[cynic(rename = "Character")]
    pub character: Option<Character>,
}

#[derive(cynic::QueryVariables, Debug, Clone)]
pub struct CharacterQuerrySearchVariables<'a> {
    pub search: Option<&'a str>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Query", variables = "CharacterQuerrySearchVariables")]
pub struct CharacterQuerrySearch {
    #[arguments(search: $ search)]
    #[cynic(rename = "Character")]
    pub character: Option<Character>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct Character {
    pub age: Option<String>,
    pub blood_type: Option<String>,
    pub date_of_birth: Option<FuzzyDate>,
    pub description: Option<String>,
    pub favourites: Option<i32>,
    pub gender: Option<String>,
    pub id: i32,
    pub image: Option<CharacterImage>,
    pub mod_notes: Option<String>,
    pub name: Option<CharacterName>,
    pub site_url: Option<String>,
    pub updated_at: Option<i32>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct CharacterName {
    pub user_preferred: Option<String>,
    pub native: Option<String>,
    pub full: Option<String>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct CharacterImage {
    pub medium: Option<String>,
    pub large: Option<String>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct FuzzyDate {
    pub month: Option<i32>,
    pub year: Option<i32>,
    pub day: Option<i32>,
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
    character: Character,
    db_type: String,
) -> Result<(), Box<dyn Error>> {
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };

    trace!("{:#?}", guild_id);

    let character_localised = load_localization_character(guild_id, db_type).await?;

    let date_of_birth_data = character.date_of_birth.clone();
    let mut fields = Vec::new();
    if let Some(date_of_birth_data) = date_of_birth_data {
        let mut has_month: bool = false;
        let mut has_day: bool = false;
        let mut date_of_birth_string = String::new();
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

        fields.push((
            character_localised.date_of_birth,
            date_of_birth_string,
            true,
        ));
    }

    let gender = character.gender.clone();
    if let Some(gender) = gender {
        fields.push((character_localised.gender, gender, true));
    }

    let age = character.age.clone();
    if let Some(age) = age {
        fields.push((character_localised.age, age, true));
    }

    let favourites = character.favourites.clone();
    if let Some(favourites) = favourites {
        fields.push((character_localised.fav, favourites.to_string(), true));
    }

    let blood_type = character.blood_type.clone();
    if let Some(blood_type) = blood_type {
        fields.push((character_localised.blood_type, blood_type, true));
    }
    let mut desc = character.description.unwrap_or_default();
    desc = convert_anilist_flavored_to_discord_flavored_markdown(desc);
    let lenght_diff = 4096 - desc.len() as i32;
    if lenght_diff <= 0 {
        desc = trim(desc, lenght_diff)
    }

    let name = match character.name.clone() {
        Some(name) => name,
        None => {
            return Err(Box::new(UnknownResponseError::Option(
                "No name found".to_string(),
            )))
        }
    };
    let native = name.native.unwrap_or_default();
    let user_pref = name.user_preferred.unwrap_or_default();
    let character_name = format!("{}/{}", user_pref, native);

    let mut builder_embed = CreateEmbed::new()
        .timestamp(Timestamp::now())
        .color(COLOR)
        .description(desc)
        .title(character_name)
        .url(character.site_url.unwrap_or_default())
        .fields(fields);
    match character.image {
        Some(image) => match image.large {
            Some(large) => builder_embed = builder_embed.thumbnail(large),
            None => {}
        },
        None => {}
    }

    let builder_message = CreateInteractionResponseMessage::new().embed(builder_embed);

    let builder = CreateInteractionResponse::Message(builder_message);

    command_interaction
        .create_response(&ctx.http, builder)
        .await
        .map_err(|e| ResponseError::Sending(format!("{:#?}", e)))?;
    Ok(())
}
