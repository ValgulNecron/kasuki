use std::error::Error;
use std::sync::Arc;

use cynic::{GraphQlResponse, QueryBuilder};
use moka::future::Cache;
use serenity::all::{
    CommandInteraction, Context, CreateInteractionResponse, CreateInteractionResponseMessage,
};
use tokio::sync::RwLock;

use crate::config::Config;
use crate::helper::create_default_embed::get_default_embed;
use crate::helper::error_management::error_enum::ResponseError;
use crate::helper::get_option::command::get_option_map_string;
use crate::helper::get_option::subcommand::get_option_map_string_subcommand;
use crate::helper::make_graphql_cached::make_request_anilist;
use crate::structure::message::anilist_user::staff::load_localization_staff;
use crate::structure::run::anilist::staff::{
    StaffQuerryId, StaffQuerryIdVariables, StaffQuerrySearch, StaffQuerrySearchVariables,
};

/// Executes the command to fetch and display information about a seiyuu (voice actor) from AniList.
///
/// This function retrieves the name or ID of the seiyuu from the command interaction and fetches the seiyuu's data from AniList.
/// It then creates a combined image of the seiyuu and the characters they have voiced, and sends this image as a response to the command interaction.
/// The function also handles errors that may occur during the execution of the command, such as errors in fetching data from AniList, creating the image, or sending the response.
///
/// # Arguments
///
/// * `ctx` - The context in which this command is being executed.
/// * `command_interaction` - The interaction that triggered this command.
///
/// # Returns
///
/// A `Result` that is `Ok` if the command executed successfully, or `Err` if an error occurred.
pub async fn run(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    config: Arc<Config>,
    anilist_cache: Arc<RwLock<Cache<String, String>>>,
) -> Result<(), Box<dyn Error>> {
    let db_type = config.bot.config.db_type.clone();
    let map = get_option_map_string(command_interaction);
    let value = map
        .get(&String::from("staff_name"))
        .ok_or(ResponseError::Option(String::from(
            "No staff name specified",
        )))?;

    let staff = if value.parse::<i32>().is_ok() {
        let var = StaffQuerryIdVariables {
            id: Some(value.parse().unwrap()),
        };
        let operation = StaffQuerryId::build(var);
        let data: GraphQlResponse<StaffQuerryId> =
            make_request_anilist(operation, false, anilist_cache).await?;
        data.data.unwrap().staff.unwrap()
    } else {
        let var = StaffQuerrySearchVariables {
            search: Some(value),
        };
        let operation = StaffQuerrySearch::build(var);
        let data: GraphQlResponse<StaffQuerrySearch> =
            make_request_anilist(operation, false, anilist_cache).await?;
        data.data.unwrap().staff.unwrap()
    };
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };

    let staff_localised = load_localization_staff(guild_id, db_type).await?;

    let mut date = String::new();
    let mut day = false;
    let mut month = false;

    let date_of_birth = staff.date_of_birth.unwrap().clone();
    if let Some(m) = date_of_birth.month {
        month = true;
        date.push_str(m.to_string().as_str())
    }

    if let Some(d) = date_of_birth.day {
        day = true;
        if month {
            date.push('/')
        }
        date.push_str(d.to_string().as_str())
    }

    if let Some(y) = date_of_birth.year {
        if day {
            date.push('/')
        }
        date.push_str(y.to_string().as_str())
    }

    let dob = staff_localised
        .date_of_birth
        .replace("$date$", date.as_str());

    let mut date = String::new();
    let mut day = false;
    let mut month = false;

    let date_of_death = staff.date_of_death.unwrap().clone();
    if let Some(m) = date_of_death.month {
        month = true;
        date.push_str(m.to_string().as_str())
    }

    if let Some(d) = date_of_death.day {
        day = true;
        if month {
            date.push('/')
        }
        date.push_str(d.to_string().as_str())
    }

    if let Some(y) = date_of_death.year {
        if day {
            date.push('/')
        }
        date.push_str(y.to_string().as_str())
    }
    let dod = staff_localised
        .date_of_death
        .replace("$date$", date.as_str());
    let desc = staff_localised
        .desc
        .replace("$dob$", dob.as_str())
        .replace("$dod$", dod.as_str())
        .replace(
            "$job$",
            staff.primary_occupations.unwrap()[0]
                .clone()
                .unwrap_or_default()
                .as_str(),
        )
        .replace(
            "$gender$",
            staff
                .gender
                .clone()
                .unwrap_or(String::from("Unknown."))
                .as_str(),
        )
        .replace("$age$", staff.age.unwrap_or(0).to_string().as_str());

    let name = staff.name.unwrap();
    let name = name.full.unwrap_or(
        name.user_preferred
            .unwrap_or(name.native.unwrap_or(String::from("Unknown."))),
    );

    let va = staff
        .characters
        .unwrap()
        .nodes
        .unwrap()
        .iter()
        .filter_map(|x| {
            let x = x.clone().unwrap();
            let name = x.name.unwrap();
            let full = name.full.as_deref();
            let native = name.native.as_deref();
            get_full_name(full, native)
        })
        .take(5)
        .collect::<Vec<String>>()
        .join("\n");

    let media = staff
        .staff_media
        .unwrap()
        .edges
        .unwrap()
        .iter()
        .filter_map(|x| {
            let node = x.clone().unwrap().node.unwrap();
            let title = node.title.unwrap();
            let romaji = title.romaji.as_deref();
            let english = title.english.as_deref();
            get_full_name(romaji, english)
        })
        .take(5)
        .collect::<Vec<String>>()
        .join("\n");

    let builder_embed = get_default_embed(None)
        .description(desc)
        .title(name)
        .url(staff.site_url.unwrap_or_default())
        .thumbnail(staff.image.unwrap().large.unwrap_or_default())
        .field(&staff_localised.field2_title, va, true)
        .field(&staff_localised.field1_title, media, true);
    let builder_message = CreateInteractionResponseMessage::new().embed(builder_embed);

    let builder = CreateInteractionResponse::Message(builder_message);

    command_interaction
        .create_response(&ctx.http, builder)
        .await
        .map_err(|e| ResponseError::Sending(format!("{:#?}", e)))?;

    Ok(())
}

/// Formats the full name of a character or staff member.
///
/// This function takes the English and native names of a character or staff member and formats them into a single string.
/// If both names are available, they are combined with a slash in between.
/// If only one name is available, that name is returned.
/// If neither name is available, `None` is returned.
///
/// # Arguments
///
/// * `a` - The English name of the character or staff member.
/// * `b` - The native name of the character or staff member.
///
/// # Returns
///
/// A `Option<String>` that contains the formatted full name of the character or staff member, or `None` if neither name is available.
fn get_full_name(a: Option<&str>, b: Option<&str>) -> Option<String> {
    match (a, b) {
        (Some(a), Some(b)) => Some(format!("{}/{}", a, b)),
        (Some(a), None) => Some(a.to_string()),
        (None, Some(b)) => Some(b.to_string()),
        (None, None) => None,
    }
}
