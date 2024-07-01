use std::sync::Arc;

use moka::future::Cache;
use serenity::all::CreateInteractionResponse::Defer;
use serenity::all::{
    CommandInteraction, Context, CreateButton, CreateEmbed, CreateInteractionResponseFollowup,
    CreateInteractionResponseMessage, PartialGuild, User, UserId,
};
use tokio::sync::RwLock;

use crate::command::run::anilist_user::user::get_user;
use crate::config::Config;
use crate::constant::{MEMBER_LIST_LIMIT, PASS_LIMIT};
use crate::database::data_struct::registered_user::RegisteredUser;
use crate::database::manage::dispatcher::data_dispatch::get_registered_user;
use crate::helper::create_default_embed::get_default_embed;
use crate::helper::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use crate::structure::message::anilist_server::list_register_user::{
    load_localization_list_user, ListUserLocalised,
};

/// This asynchronous function runs the command interaction for listing registered AniList users in a Discord guild.
///
/// It first retrieves the guild ID from the command interaction. If the command interaction does not have a guild ID, it uses "0" as the guild ID.
///
/// It loads the localized text for the list user command.
///
/// It sends a deferred response to the command interaction.
///
/// It retrieves the guild from the guild ID.
///
/// It retrieves a list of AniList users in the guild by calling the `get_the_list` function.
///
/// It checks if the number of AniList users is greater than the limit. If it is, it adds a "next" button to the response.
///
/// It sends a followup message with the list of AniList users.
///
/// It handles any errors that occur during the process and returns an `AppError` if an error occurs.
///
/// # Arguments
///
/// * `ctx` - The context in which this function is being called.
/// * `command_interaction` - The command interaction that triggered this function.
///
/// # Returns
///
/// A `Result` indicating whether the function executed successfully. If an error occurred, it contains an `AppError`.
pub async fn run(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    config: Arc<Config>,
    anilist_cache: Arc<RwLock<Cache<String, String>>>,
) -> Result<(), AppError> {
    let db_type = config.bot.config.db_type.clone();
    // Retrieve the guild ID from the command interaction or use "0" if it does not exist
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };

    // Load the localized text for the list user command
    let list_user_localised = load_localization_list_user(guild_id, db_type.clone()).await?;

    // Retrieve the guild from the guild ID
    let guild_id = command_interaction.guild_id.ok_or(AppError::new(
        String::from("There is no guild id"),
        ErrorType::Option,
        ErrorResponseType::Message,
    ))?;

    let guild = guild_id
        .to_partial_guild_with_counts(&ctx.http)
        .await
        .map_err(|e| {
            AppError::new(
                format!("Error while getting the guild {}", e),
                ErrorType::WebRequest,
                ErrorResponseType::Message,
            )
        })?;

    // Send a deferred response to the command interaction
    let builder_message = Defer(CreateInteractionResponseMessage::new());

    command_interaction
        .create_response(&ctx.http, builder_message)
        .await
        .map_err(|e| {
            AppError::new(
                format!("Error while sending the command {}", e),
                ErrorType::Command,
                ErrorResponseType::Message,
            )
        })?;

    // Retrieve a list of AniList users in the guild
    let (builder_message, len, last_id): (CreateEmbed, usize, Option<UserId>) = get_the_list(
        guild,
        ctx,
        &list_user_localised,
        None,
        db_type,
        anilist_cache,
    )
    .await?;

    // Check if the number of AniList users is greater than the limit
    let mut response = CreateInteractionResponseFollowup::new().embed(builder_message);
    if len >= (MEMBER_LIST_LIMIT + 1) as usize {
        // If the number of AniList users is greater than the limit, add a "next" button to the response
        response = response.button(
            CreateButton::new(format!("user_{}_0", last_id.unwrap()))
                .label(&list_user_localised.next),
        )
    }

    // Send a followup message with the list of AniList users
    command_interaction
        .create_followup(&ctx.http, response)
        .await
        .map_err(|e| {
            AppError::new(
                format!("Error while sending the command {}", e),
                ErrorType::Command,
                ErrorResponseType::Followup,
            )
        })?;
    Ok(())
}

/// Data structure for storing a Discord user and their corresponding AniList user.
struct Data {
    pub user: User,                                          // The Discord user.
    pub anilist: crate::structure::run::anilist::user::User, // The corresponding AniList user.
}

/// This asynchronous function retrieves a list of AniList users in a Discord guild.
///
/// It retrieves the members of the guild and checks if they are registered AniList users.
/// If they are, it adds them to a list.
///
/// It continues retrieving members and checking if they are registered AniList users until it has retrieved a certain number of AniList users or it has checked a certain number of members.
///
/// It then formats the list of AniList users into a string and returns it along with the number of AniList users and the ID of the last member checked.
///
/// # Arguments
///
/// * `guild` - The Discord guild to retrieve the AniList users from.
/// * `ctx` - The context in which this function is being called.
/// * `list_user_localised` - The localized text for the list user command.
/// * `last_id` - The ID of the last member to start retrieving members from.
///
/// # Returns
///
/// A `Result` containing a tuple with the formatted list of AniList users, the number of AniList users, and the ID of the last member checked. If an error occurred, it contains an `AppError`.
pub async fn get_the_list(
    guild: PartialGuild,
    ctx: &Context,
    list_user_localised: &ListUserLocalised,
    last_id: Option<UserId>,
    db_type: String,
    anilist_cache: Arc<RwLock<Cache<String, String>>>,
) -> Result<(CreateEmbed, usize, Option<UserId>), AppError> {
    let mut anilist_user = Vec::new();
    let mut last_id: Option<UserId> = last_id;
    let mut pass = 0;
    while anilist_user.len() < MEMBER_LIST_LIMIT as usize && pass < PASS_LIMIT {
        let members = guild
            .members(&ctx.http, Some(MEMBER_LIST_LIMIT), last_id)
            .await
            .map_err(|e| {
                AppError::new(
                    format!("Error while getting the members {}", e),
                    ErrorType::WebRequest,
                    ErrorResponseType::Followup,
                )
            })?;

        for member in members {
            last_id = Some(member.user.id);
            let user_id = member.user.id.to_string();
            let row: Option<RegisteredUser> =
                get_registered_user(&user_id, db_type.clone()).await?;
            let user_data = match row {
                Some(a) => get_user(&a.anilist_id, anilist_cache.clone()).await?,
                None => continue,
            };
            let data = Data {
                user: member.user,
                anilist: user_data,
            };
            anilist_user.push(data)
        }
        pass += 1
    }

    let user_links: Vec<String> = anilist_user
        .iter()
        .map(|data| {
            format!(
                "[{}](<https://anilist_user.co/user/{}>)",
                data.user.name, data.anilist.id
            )
        })
        .collect();
    let joined_string = user_links.join("\n\n");

    Ok((
        get_default_embed(None)
            .title(&list_user_localised.title)
            .description(joined_string),
        anilist_user.len(),
        last_id,
    ))
}
