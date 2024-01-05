use crate::common::steam_to_discord_markdown::convert_steam_to_discord_flavored_markdown;
use crate::constant::{COLOR, COMMAND_SENDING_ERROR, OPTION_ERROR};
use crate::error_enum::AppError;
use crate::game_struct::run::steam_game::SteamGameWrapper;
use crate::lang_struct::game::steam_game_info::load_localization_steam_game_info;
use serenity::all::{
    CommandDataOption, CommandInteraction, Context, CreateEmbed, CreateInteractionResponse,
    CreateInteractionResponseMessage, Timestamp,
};

pub async fn run(
    options: &[CommandDataOption],
    ctx: &Context,
    command_interaction: &CommandInteraction,
) -> Result<(), AppError> {
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };
    for option in options {
        if option.name.as_str() != "type" {
            if let Some(a) = option.value.as_str() {
                let value = &a.to_string();

                let data: SteamGameWrapper = if value.parse::<i128>().is_ok() {
                    SteamGameWrapper::new_steam_game_by_id(value.parse().unwrap(), guild_id).await?
                } else {
                    SteamGameWrapper::new_steam_game_by_search(value, guild_id).await?
                };
                return send_embed(ctx, command_interaction, data).await;
            }
        }
    }

    Err(OPTION_ERROR.clone())
}

async fn send_embed(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    data: SteamGameWrapper,
) -> Result<(), AppError> {
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };

    let steam_game_info_localised = load_localization_steam_game_info(guild_id).await?;

    let game = data.data;

    let mut fields = Vec::new();

    let field1 = if game.is_free.unwrap() {
        (
            steam_game_info_localised.field1,
            steam_game_info_localised.free,
            true,
        )
    } else {
        (
            steam_game_info_localised.field1,
            game.price_overview.unwrap().final_formatted.unwrap(),
            true,
        )
    };
    let field2 = (
        steam_game_info_localised.field2,
        game.release_date.unwrap().date.unwrap(),
        true,
    );
    let field3 = (
        steam_game_info_localised.field3,
        game.developers.unwrap().join(", "),
        true,
    );
    let field4 = (
        steam_game_info_localised.field4,
        game.publishers.unwrap().join(", "),
        true,
    );
    let descriptions: Vec<String> = game
        .categories
        .unwrap()
        .into_iter()
        .filter_map(|category| category.description)
        .collect();

    let joined_descriptions = descriptions.join(" | ");
    let field5 = (steam_game_info_localised.field5, joined_descriptions, true);

    fields.push(field1);
    fields.push(field2);
    fields.push(field3);
    fields.push(field4);
    fields.push(field5);

    let builder_embed = CreateEmbed::new()
        .timestamp(Timestamp::now())
        .color(COLOR)
        .title(game.name.unwrap())
        .description(convert_steam_to_discord_flavored_markdown(
            game.short_description.unwrap(),
        ))
        .fields(fields)
        .url(format!(
            "https://store.steampowered.com/app/{}",
            game.steam_appid.unwrap()
        ))
        .image(game.header_image.unwrap());
    let builder_message = CreateInteractionResponseMessage::new().embed(builder_embed);

    let builder = CreateInteractionResponse::Message(builder_message);

    command_interaction
        .create_response(&ctx.http, builder)
        .await
        .map_err(|_| COMMAND_SENDING_ERROR.clone())
}
