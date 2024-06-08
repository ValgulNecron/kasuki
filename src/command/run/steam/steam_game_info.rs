use serenity::all::CreateInteractionResponse::Defer;
use serenity::all::{
    CommandInteraction, Context, CreateInteractionResponseFollowup,
    CreateInteractionResponseMessage,
};
use tracing::trace;

use crate::helper::convert_flavored_markdown::convert_steam_to_discord_flavored_markdown;
use crate::helper::create_default_embed::get_default_embed;
use crate::helper::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use crate::helper::get_option::subcommand::get_option_map_string_subcommand;
use crate::structure::message::game::steam_game_info::{
    load_localization_steam_game_info, SteamGameInfoLocalised,
};
use crate::structure::run::game::steam_game::SteamGameWrapper;

/// Executes the command to retrieve and display a Steam game's information.
///
/// This function retrieves the game's name from the command interaction, loads the localized strings for the game's information,
/// creates a deferred response to the command interaction, retrieves the game's information from Steam, and sends an embed with the game's information.
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
    // Retrieve the game's name from the command interaction
    let map = get_option_map_string_subcommand(command_interaction);
    let value = map.get(&String::from("game_name")).ok_or(AppError::new(
        String::from("There is no option"),
        ErrorType::Option,
        ErrorResponseType::Followup,
    ))?;

    // Retrieve the guild ID from the command interaction or use "0" if it does not exist
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };

    // Load the localized strings for the game's information
    let steam_game_info_localised = load_localization_steam_game_info(guild_id.clone()).await?;

    // Create a deferred response to the command interaction
    let builder_message = Defer(CreateInteractionResponseMessage::new());

    // Send the deferred response
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

    // Retrieve the game's information from Steam
    let data: SteamGameWrapper = if value.parse::<i128>().is_ok() {
        SteamGameWrapper::new_steam_game_by_id(value.parse().unwrap(), guild_id).await?
    } else {
        SteamGameWrapper::new_steam_game_by_search(value, guild_id).await?
    };

    // Send an embed with the game's information
    send_embed(ctx, command_interaction, data, steam_game_info_localised).await
}

/// Sends an embed with the Steam game's information.
///
/// This function retrieves the Steam game's information and formats them into a response to the command interaction.
/// The response includes the game's price, release date, developers, publishers, app type, supported languages, and categories, which are sent as an embed.
///
/// # Arguments
///
/// * `ctx` - The context in which this command is being executed.
/// * `command_interaction` - The interaction that triggered this command.
/// * `data` - The Steam game's information wrapped in a `SteamGameWrapper`.
/// * `steam_game_info_localised` - The localized strings for the Steam game's information.
///
/// # Returns
///
/// A `Result` that is `Ok` if the command executed successfully, or `Err` if an error occurred.
async fn send_embed(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    data: SteamGameWrapper,
    steam_game_info_localised: SteamGameInfoLocalised,
) -> Result<(), AppError> {
    trace!("Sending embed.");
    let game = data.data;

    let mut fields = Vec::new();

    // Determine the price field based on whether the game is free or not
    let field1 = if game.is_free.unwrap() {
        (
            steam_game_info_localised.field1,
            steam_game_info_localised.free,
            true,
        )
    } else {
        match game.price_overview {
            Some(price) => (
                steam_game_info_localised.field1,
                convert_steam_to_discord_flavored_markdown(price.final_formatted.unwrap()),
                true,
            ),
            None => (
                steam_game_info_localised.field1,
                steam_game_info_localised.tba,
                true,
            ),
        }
    };
    fields.push(field1);

    // Determine the release date field based on whether the game is coming soon or not
    let field2 = if game.release_date.clone().unwrap().coming_soon {
        match game.release_date.unwrap().date {
            Some(date) => (
                steam_game_info_localised.field2,
                convert_steam_to_discord_flavored_markdown(date),
                true,
            ),
            None => (
                steam_game_info_localised.field2,
                steam_game_info_localised.coming_soon,
                true,
            ),
        }
    } else {
        (
            steam_game_info_localised.field2,
            convert_steam_to_discord_flavored_markdown(game.release_date.unwrap().date.unwrap()),
            true,
        )
    };
    fields.push(field2);

    // Add the developers field if it exists
    if let Some(dev) = game.developers {
        fields.push((
            steam_game_info_localised.field3,
            convert_steam_to_discord_flavored_markdown(dev.join(", ")),
            true,
        ))
    }

    // Add the publishers field if it exists
    if let Some(publishers) = game.publishers {
        fields.push((
            steam_game_info_localised.field4,
            convert_steam_to_discord_flavored_markdown(publishers.join(", ")),
            true,
        ))
    }

    // Add the app type field if it exists
    if let Some(app_type) = game.app_type {
        fields.push((
            steam_game_info_localised.field5,
            convert_steam_to_discord_flavored_markdown(app_type),
            true,
        ))
    }

    // Add the supported languages field if it exists
    if let Some(game_lang) = game.supported_languages {
        fields.push((
            steam_game_info_localised.field6,
            convert_steam_to_discord_flavored_markdown(game_lang),
            false,
        ))
    }

    // Add the categories field if it exists
    if let Some(categories) = game.categories {
        let descriptions: Vec<String> = categories
            .into_iter()
            .filter_map(|category| category.description)
            .collect();
        let joined_descriptions =
            convert_steam_to_discord_flavored_markdown(descriptions.join(", "));
        fields.push((steam_game_info_localised.field7, joined_descriptions, false))
    }

    // Construct the embed for the response
    let builder_embed = get_default_embed(None)
        .title(game.name.unwrap())
        .description(convert_steam_to_discord_flavored_markdown(
            game.short_description.unwrap_or_default(),
        ))
        .fields(fields)
        .url(format!(
            "https://store.steampowered.com/app/{}",
            game.steam_appid.unwrap()
        ))
        .image(game.header_image.unwrap());
    let builder_message = CreateInteractionResponseFollowup::new().embed(builder_embed);

    // Send the follow-up response to the command interaction
    command_interaction
        .create_followup(&ctx.http, builder_message)
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
