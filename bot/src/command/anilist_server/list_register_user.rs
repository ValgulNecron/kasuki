use std::error::Error;
use std::sync::Arc;

use crate::command::command_trait::{Command, SlashCommand};
use crate::config::{BotConfigDetails, Config};
use crate::constant::{MEMBER_LIST_LIMIT, PASS_LIMIT};
use crate::get_url;
use crate::helper::create_default_embed::get_default_embed;
use crate::helper::error_management::error_enum::{FollowupError, ResponseError};
use crate::structure::database::prelude::RegisteredUser;
use crate::structure::database::registered_user::{Column, Model};
use crate::structure::message::anilist_server::list_register_user::{
    load_localization_list_user, ListUserLocalised,
};
use moka::future::Cache;
use sea_orm::ColumnTrait;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use serenity::all::CreateInteractionResponse::Defer;
use serenity::all::{
    CommandInteraction, Context, CreateButton, CreateEmbed, CreateInteractionResponseFollowup,
    CreateInteractionResponseMessage, PartialGuild, User, UserId,
};
use tokio::sync::RwLock;

pub struct ListRegisterUser {
    pub ctx: Context,
    pub command_interaction: CommandInteraction,
    pub config: Arc<Config>,
    pub anilist_cache: Arc<RwLock<Cache<String, String>>>,
}

impl Command for ListRegisterUser {
    fn get_ctx(&self) -> &Context {
        &self.ctx
    }
    fn get_command_interaction(&self) -> &CommandInteraction {
        &self.command_interaction
    }
}

impl SlashCommand for ListRegisterUser {
    async fn run_slash(&self) -> Result<(), Box<dyn Error>> {
        send_embed(
            &self.ctx,
            &self.command_interaction,
            self.config.clone(),
            self.anilist_cache.clone(),
        )
        .await
    }
}
async fn send_embed(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    config: Arc<Config>,
) -> Result<(), Box<dyn Error>> {
    // Retrieve the guild ID from the command interaction or use "0" if it does not exist
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };

    // Load the localized text for the list user command
    let list_user_localised =
        load_localization_list_user(guild_id, config.bot.config.clone()).await?;

    // Retrieve the guild from the guild ID
    let guild_id = command_interaction
        .guild_id
        .ok_or(ResponseError::Option(String::from(
            "Could not get the id of the guild",
        )))?;

    let guild = guild_id
        .to_partial_guild_with_counts(&ctx.http)
        .await
        .map_err(|e| ResponseError::UserOrGuild(format!("{:#?}", e)))?;

    // Send a deferred response to the command interaction
    let builder_message = Defer(CreateInteractionResponseMessage::new());

    command_interaction
        .create_response(&ctx.http, builder_message)
        .await
        .map_err(|e| ResponseError::Sending(format!("{:#?}", e)))?;

    // Retrieve a list of AniList users in the guild
    let (builder_message, len, last_id): (CreateEmbed, usize, Option<UserId>) = get_the_list(
        guild,
        ctx,
        &list_user_localised,
        None,
        config.bot.config.clone(),
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
        .map_err(|e| FollowupError::Sending(format!("{:#?}", e)))?;
    Ok(())
}

/// Data structure for storing a Discord user and their corresponding AniList user.
struct Data {
    pub user: User,      // The Discord user.
    pub anilist: String, // The corresponding AniList user.
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
    db_config: BotConfigDetails,
) -> Result<(CreateEmbed, usize, Option<UserId>), Box<dyn Error>> {
    let mut anilist_user = Vec::new();
    let mut last_id: Option<UserId> = last_id;
    let mut pass = 0;
    while anilist_user.len() < MEMBER_LIST_LIMIT as usize && pass < PASS_LIMIT {
        pass += 1;
        let members = guild
            .members(&ctx.http, Some(MEMBER_LIST_LIMIT), last_id)
            .await
            .map_err(|e| FollowupError::UserOrGuild(format!("{:#?}", e)))?;
        if members.is_empty() {
            break;
        }

        for member in members {
            last_id = Some(member.user.id);
            let user_id = member.user.id.to_string();
            let connection = sea_orm::Database::connect(get_url(db_config.clone())).await?;
            let row = RegisteredUser::find()
                .filter(Column::UserId.eq(user_id.clone()))
                .one(&connection)
                .await?
                .unwrap_or(Model {
                    user_id: user_id.clone(),
                    anilist_id: 2134,
                    registered_at: Default::default(),
                });
            let user_data = row;
            let data = Data {
                user: member.user,
                anilist: user_data.anilist_id.to_string(),
            };
            anilist_user.push(data)
        }
    }

    let user_links: Vec<String> = anilist_user
        .iter()
        .map(|data| {
            format!(
                "[{}](<https://anilist_user.co/user/{}>)",
                data.user.name, data.anilist
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
