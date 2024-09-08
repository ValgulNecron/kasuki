use std::error::Error;

use crate::config::DbConfig;
use serde::{Deserialize, Serialize};

// Importing necessary libraries and modules
use crate::structure::message::common::load_localization;

/// StaffLocalised struct represents a staff's localized data.
/// It contains fields for two titles, description, date of birth, and date of death.
///
/// # Fields
/// * `field1_title`: A string representing the first title related data.
/// * `field2_title`: A string representing the second title related data.
/// * `desc`: A string representing the description related data.
/// * `date_of_birth`: A string representing the date of birth related data.
/// * `date_of_death`: A string representing the date of death related data.
#[derive(Debug, Deserialize, Serialize, Clone)]

pub struct StaffLocalised {
    pub field1_title: String,
    pub field2_title: String,
    pub desc: String,
    pub date_of_birth: String,
    pub date_of_death: String,
}

/// This function loads the localization data for a staff.
/// It takes a guild_id as input and returns a Result containing StaffLocalised data or an AppError.
/// The function reads a JSON file, parses it into a HashMap, and then retrieves the data based on the guild's language.
///
/// # Arguments
///
/// * `guild_id`: A string representing the guild id.
///
/// # Returns
///
/// * `Result<StaffLocalised, AppError>`: A Result containing StaffLocalised data or an AppError.

pub async fn load_localization_staff(
    guild_id: String,
    db_config: DbConfig,
) -> Result<StaffLocalised, Box<dyn Error>> {

    let path = "json/message/anilist_user/staff.json";

    load_localization(guild_id, path, db_config).await
}
