use serenity::all::{CommandInteraction, Context};

use crate::database::data_struct::registered_user::RegisteredUser;
use crate::database::manage::dispatcher::data_dispatch::get_registered_user;
use crate::helper::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use crate::helper::get_option::subcommand::get_option_map_string_subcommand;
use crate::structure::run::anilist::user::{send_embed, UserWrapper};

/// Executes the command to fetch and display information about a user from AniList.
///
/// This function retrieves the username from the command interaction and fetches the user's data from AniList.
/// If the username is not provided, it fetches the data of the user who triggered the command interaction.
/// It then sends the user's data as a response to the command interaction.
///
/// # Arguments
///
/// * `ctx` - The context in which this command is being executed.
/// * `command_interaction` - The interaction that triggered this command.
///
/// # Returns
///
/// A `Result` that is `Ok` if the command executed successfully, or `Err` if an error occurred.
pub async fn run(ctx: &Context, command_interaction: &CommandInteraction) -> Result<(), AppError> {
    // Retrieve the username from the command interaction
    let map = get_option_map_string_subcommand(command_interaction);
    let user = map.get(&String::from("username"));

    // If the username is provided, fetch the user's data from AniList and send it as a response
    if let Some(value) = user {
        let data: UserWrapper = get_user_data(value).await?;
        return send_embed(ctx, command_interaction, data).await;
    }

    // If the username is not provided, fetch the data of the user who triggered the command interaction
    let user_id = &command_interaction.user.id.to_string();
    let row: Option<RegisteredUser> = get_registered_user(user_id).await?;
    let user = row.ok_or(AppError::new(
        String::from("There is no option"),
        ErrorType::Option,
        ErrorResponseType::Followup,
    ))?;

    // Fetch the user's data from AniList and send it as a response
    let data = get_user_data(&user.anilist_id).await?;
    send_embed(ctx, command_interaction, data).await
}

/// Fetches the data of a user from AniList.
///
/// This function takes a username or user ID and fetches the user's data from AniList.
/// If the username or user ID is not valid, it returns an error.
///
/// # Arguments
///
/// * `value` - The username or user ID of the user.
///
/// # Returns
///
/// A `Result` that is `Ok` if the user's data was fetched successfully, or `Err` if an error occurred.
pub async fn get_user_data(value: &String) -> Result<UserWrapper, AppError> {
    // If the value is a valid user ID, fetch the user's data by ID
    if value.parse::<i32>().is_ok() {
        UserWrapper::new_user_by_id(value.parse().unwrap()).await
    } else {
        // If the value is not a valid user ID, fetch the user's data by username
        UserWrapper::new_user_by_search(value).await
    }
}
