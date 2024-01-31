use crate::common::steam_to_discord_markdown::convert_steam_to_discord_flavored_markdown;
use crate::constant::COLOR;
use crate::error_enum::AppError;
use crate::error_enum::AppError::{CommandSendingError, OptionError};
use crate::game_struct::run::steam_game::SteamGameWrapper;
use crate::lang_struct::game::steam_game_info::{
    load_localization_steam_game_info, SteamGameInfoLocalised,
};
use serenity::all::CreateInteractionResponse::Defer;
use serenity::all::{
    CommandDataOption, CommandInteraction, Context, CreateEmbed, CreateInteractionResponseFollowup,
    CreateInteractionResponseMessage, Timestamp,
};
use tracing::trace;

pub async fn run(
    options: &[CommandDataOption],
    ctx: &Context,
    command_interaction: &CommandInteraction,
) -> Result<(), AppError> {
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };

    let steam_game_info_localised = load_localization_steam_game_info(guild_id.clone()).await?;

    let builder_message = Defer(CreateInteractionResponseMessage::new());

    command_interaction
        .create_response(&ctx.http, builder_message)
        .await
        .map_err(|e| CommandSendingError(format!("Error while sending the command {}", e)))?;

    for option in options {
        if option.name.as_str() != "type" {
            if let Some(a) = option.value.as_str() {
                let value = &a.to_string();

                let data: SteamGameWrapper = if value.parse::<i128>().is_ok() {
                    SteamGameWrapper::new_steam_game_by_id(value.parse().unwrap(), guild_id).await?
                } else {
                    SteamGameWrapper::new_steam_game_by_search(value, guild_id).await?
                };
                return send_embed(ctx, command_interaction, data, steam_game_info_localised).await;
            }
        }
    }

    Err(OptionError(String::from("There is no option")))
}

async fn send_embed(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    data: SteamGameWrapper,
    steam_game_info_localised: SteamGameInfoLocalised,
) -> Result<(), AppError> {
    trace!("Sending embed.");
    let game = data.data;

    let mut fields = Vec::new();

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

    if let Some(dev) = game.developers {
        fields.push((
            steam_game_info_localised.field3,
            convert_steam_to_discord_flavored_markdown(dev.join(", ")),
            true,
        ))
    }

    if let Some(publishers) = game.publishers {
        fields.push((
            steam_game_info_localised.field4,
            convert_steam_to_discord_flavored_markdown(publishers.join(", ")),
            true,
        ))
    }

    if let Some(app_type) = game.app_type {
        fields.push((
            steam_game_info_localised.field5,
            convert_steam_to_discord_flavored_markdown(app_type),
            true,
        ))
    }

    if let Some(game_lang) = game.supported_languages {
        fields.push((
            steam_game_info_localised.field6,
            convert_steam_to_discord_flavored_markdown(game_lang),
            false,
        ))
    }

    if let Some(categories) = game.categories {
        let descriptions: Vec<String> = categories
            .into_iter()
            .filter_map(|category| category.description)
            .collect();
        let joined_descriptions =
            convert_steam_to_discord_flavored_markdown(descriptions.join(", "));
        fields.push((steam_game_info_localised.field7, joined_descriptions, false))
    }

    let builder_embed = CreateEmbed::new()
        .timestamp(Timestamp::now())
        .color(COLOR)
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

    let _ = command_interaction
        .create_followup(&ctx.http, builder_message)
        .await
        .map_err(|e| CommandSendingError(format!("Error while sending the command {}", e)))?;

    Ok(())
}
