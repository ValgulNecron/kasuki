use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;

use crate::command::command_trait::{Command, SlashCommand};
use crate::config::Config;
use crate::helper::convert_flavored_markdown::convert_steam_to_discord_flavored_markdown;
use crate::helper::create_default_embed::get_default_embed;
use crate::helper::error_management::error_dispatch;
use crate::helper::get_option::subcommand::get_option_map_string_subcommand;
use crate::structure::message::game::steam_game_info::load_localization_steam_game_info;
use crate::structure::run::game::steam_game::{Platforms, SteamGameWrapper};
use serenity::all::{CommandInteraction, Context, CreateInteractionResponseFollowup, GuildId};
use tokio::sync::RwLock;

pub struct SteamGameInfoCommand {
    pub ctx: Context,
    pub command_interaction: CommandInteraction,
    pub config: Arc<Config>,
    pub apps: Arc<RwLock<HashMap<String, u128>>>,
}

impl Command for SteamGameInfoCommand {
    fn get_ctx(&self) -> &Context {
        &self.ctx
    }

    fn get_command_interaction(&self) -> &CommandInteraction {
        &self.command_interaction
    }
}

impl SlashCommand for SteamGameInfoCommand {
    async fn run_slash(&self) -> Result<(), Box<dyn Error>> {
        let data = get_steam_game(
            self.apps.clone(),
            self.command_interaction.clone(),
            self.config.clone(),
        )
        .await?;
        send_embed(
            &self.ctx,
            &self.command_interaction,
            data,
            self.config.clone(),
        )
        .await
    }
}

async fn get_steam_game(
    apps: Arc<RwLock<HashMap<String, u128>>>,
    command_interaction: CommandInteraction,
    config: Arc<Config>,
) -> Result<SteamGameWrapper, Box<dyn Error>> {
    let guild_id = command_interaction
        .guild_id
        .unwrap_or(GuildId::from(0))
        .to_string();
    let map = get_option_map_string_subcommand(&command_interaction);
    let value = map
        .get(&String::from("game_name"))
        .ok_or(error_dispatch::Error::Option(String::from(
            "No option for game_name",
        )))?;
    let data: SteamGameWrapper = if value.parse::<i128>().is_ok() {
        SteamGameWrapper::new_steam_game_by_id(value.parse().unwrap(), guild_id, config.db.clone())
            .await?
    } else {
        SteamGameWrapper::new_steam_game_by_search(value, guild_id, apps, config.db.clone()).await?
    };

    Ok(data)
}

async fn send_embed(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    data: SteamGameWrapper,
    config: Arc<Config>,
) -> Result<(), Box<dyn Error>> {
    let guild_id = command_interaction
        .guild_id
        .unwrap_or(GuildId::from(0))
        .to_string();

    let steam_game_info_localised =
        load_localization_steam_game_info(guild_id.clone(), config.db.clone()).await?;

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
            Some(price) => {
                let price = format!(
                    "{} {}",
                    price.final_formatted.unwrap_or_default(),
                    price.discount_percent.unwrap_or_default()
                );
                (
                    steam_game_info_localised.field1,
                    convert_steam_to_discord_flavored_markdown(price),
                    true,
                )
            }
            None => (
                steam_game_info_localised.field1,
                steam_game_info_localised.tba,
                true,
            ),
        }
    };
    fields.push(field1);
    let platforms = match game.platforms {
        Some(platforms) => platforms,
        _ => Platforms {
            windows: None,
            mac: None,
            linux: None,
        },
    };

    if let Some(website) = game.website {
        fields.push((
            steam_game_info_localised.website,
            convert_steam_to_discord_flavored_markdown(website),
            true,
        ));
    }
    if let Some(required_age) = game.required_age {
        fields.push((
            steam_game_info_localised.required_age,
            required_age.to_string(),
            true,
        ));
    }

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
            true,
        ))
    }
    let win = platforms.windows.unwrap_or(false);
    let mac = platforms.mac.unwrap_or(false);
    let linux = platforms.linux.unwrap_or(false);
    fields.push((steam_game_info_localised.win, win.to_string(), true));
    fields.push((steam_game_info_localised.mac, mac.to_string(), true));
    fields.push((steam_game_info_localised.linux, linux.to_string(), true));

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
        .await?;

    Ok(())
}
