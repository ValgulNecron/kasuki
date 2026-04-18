use crate::command::context::CommandContext;
use crate::command::embed_content::{EmbedContent, EmbedsContents};
use crate::helper::convert_flavored_markdown::convert_steam_to_discord_flavored_markdown;
use crate::helper::get_option::subcommand::get_option_map_string_subcommand;
use crate::structure::run::game::steam_game::{Platforms, SteamGameWrapper};
use crate::structure::steam_game_index::SteamGameIndex;
use anyhow::{anyhow, Result};
use arc_swap::ArcSwap;
use kasuki_macros::slash_command;
use sea_orm::DatabaseConnection;
use serenity::all::{CommandInteraction, Context as SerenityContext, GuildId};
use shared::cache::CacheInterface;
use shared::localization::{Loader, USABLE_LOCALES};
use std::sync::Arc;

#[slash_command(
	name = "game", desc = "Get info of a steam game.",
	command_type = SubCommand(parent = "steam"),
	contexts = [Guild, BotDm, PrivateChannel],
	install_contexts = [Guild, User],
	args = [(name = "game_name", desc = "Name of the steam game you want info of.", arg_type = String, required = true, autocomplete = true)],
)]
async fn steam_game_info_command(self_: SteamGameInfoCommand) -> Result<EmbedsContents<'_>> {
	let cx = CommandContext::new(
		self_.get_ctx().clone(),
		self_.get_command_interaction().clone(),
	);
	let data = get_steam_game(
		cx.bot_data.apps.clone(),
		cx.command_interaction.clone(),
		cx.db.clone(),
		cx.bot_data.steam_cache.clone(),
	)
	.await?;

	let lang_id = cx.lang_id().await;

	let game = data.data;

	let mut fields = Vec::new();

	let field1 = if game.is_free.unwrap_or(false) {
		(
			USABLE_LOCALES.lookup(&lang_id, "game_steam_game_info-field1"),
			USABLE_LOCALES.lookup(&lang_id, "game_steam_game_info-free"),
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
					USABLE_LOCALES.lookup(&lang_id, "game_steam_game_info-field1"),
					convert_steam_to_discord_flavored_markdown(price),
					true,
				)
			},
			None => (
				USABLE_LOCALES.lookup(&lang_id, "game_steam_game_info-field1"),
				USABLE_LOCALES.lookup(&lang_id, "game_steam_game_info-tba"),
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
			USABLE_LOCALES.lookup(&lang_id, "game_steam_game_info-website"),
			convert_steam_to_discord_flavored_markdown(website),
			true,
		));
	}

	if let Some(required_age) = game.required_age {
		fields.push((
			USABLE_LOCALES.lookup(&lang_id, "game_steam_game_info-required_age"),
			required_age.to_string(),
			true,
		));
	}

	let field2 = match game.release_date {
		Some(ref release_date) if release_date.coming_soon => match &release_date.date {
			Some(date) => (
				USABLE_LOCALES.lookup(&lang_id, "game_steam_game_info-field2"),
				convert_steam_to_discord_flavored_markdown(date.clone()),
				true,
			),
			None => (
				USABLE_LOCALES.lookup(&lang_id, "game_steam_game_info-field2"),
				USABLE_LOCALES.lookup(&lang_id, "game_steam_game_info-coming_soon"),
				true,
			),
		},
		Some(ref release_date) => (
			USABLE_LOCALES.lookup(&lang_id, "game_steam_game_info-field2"),
			convert_steam_to_discord_flavored_markdown(
				release_date.date.clone().unwrap_or_default(),
			),
			true,
		),
		None => (
			USABLE_LOCALES.lookup(&lang_id, "game_steam_game_info-field2"),
			USABLE_LOCALES.lookup(&lang_id, "game_steam_game_info-coming_soon"),
			true,
		),
	};

	fields.push(field2);

	if let Some(dev) = game.developers {
		fields.push((
			USABLE_LOCALES.lookup(&lang_id, "game_steam_game_info-field3"),
			convert_steam_to_discord_flavored_markdown(dev.join(", ")),
			true,
		))
	}

	if let Some(publishers) = game.publishers {
		fields.push((
			USABLE_LOCALES.lookup(&lang_id, "game_steam_game_info-field4"),
			convert_steam_to_discord_flavored_markdown(publishers.join(", ")),
			true,
		))
	}

	if let Some(app_type) = game.app_type {
		fields.push((
			USABLE_LOCALES.lookup(&lang_id, "game_steam_game_info-field5"),
			convert_steam_to_discord_flavored_markdown(app_type),
			true,
		))
	}

	if let Some(game_lang) = game.supported_languages {
		fields.push((
			USABLE_LOCALES.lookup(&lang_id, "game_steam_game_info-field6"),
			convert_steam_to_discord_flavored_markdown(game_lang),
			true,
		))
	}

	let win = platforms.windows.unwrap_or(false);

	let mac = platforms.mac.unwrap_or(false);

	let linux = platforms.linux.unwrap_or(false);

	fields.push((
		USABLE_LOCALES.lookup(&lang_id, "game_steam_game_info-win"),
		win.to_string(),
		true,
	));

	fields.push((
		USABLE_LOCALES.lookup(&lang_id, "game_steam_game_info-mac"),
		mac.to_string(),
		true,
	));

	fields.push((
		USABLE_LOCALES.lookup(&lang_id, "game_steam_game_info-linux"),
		linux.to_string(),
		true,
	));

	if let Some(categories) = game.categories {
		let descriptions: Vec<String> = categories
			.into_iter()
			.filter_map(|category| category.description)
			.collect();

		let joined_descriptions =
			convert_steam_to_discord_flavored_markdown(descriptions.join(", "));

		fields.push((
			USABLE_LOCALES.lookup(&lang_id, "game_steam_game_info-field7"),
			joined_descriptions,
			false,
		))
	}

	let embed_content = EmbedContent::new(game.name.unwrap_or_default())
		.description(convert_steam_to_discord_flavored_markdown(
			game.short_description.unwrap_or_default(),
		))
		.fields(fields)
		.url(format!(
			"https://store.steampowered.com/app/{}",
			game.steam_appid.unwrap_or(0)
		))
		.images_url(game.header_image.unwrap_or_default());

	let embed_contents = EmbedsContents::new(vec![embed_content]);

	Ok(embed_contents)
}

async fn get_steam_game(
	apps: Arc<ArcSwap<SteamGameIndex>>, command_interaction: CommandInteraction,
	db_connection: Arc<DatabaseConnection>, steam_cache: Arc<CacheInterface>,
) -> Result<SteamGameWrapper> {
	let guild_id = command_interaction
		.guild_id
		.unwrap_or(GuildId::from(0))
		.to_string();

	let map = get_option_map_string_subcommand(&command_interaction);

	let value = map
		.get("game_name")
		.ok_or(anyhow!("No option for game_name"))?;

	let data: SteamGameWrapper = if let Ok(appid) = value.parse::<u32>() {
		SteamGameWrapper::new_steam_game_by_id(appid, guild_id, db_connection, steam_cache).await?
	} else {
		SteamGameWrapper::new_steam_game_by_search(
			value,
			guild_id,
			apps,
			db_connection,
			steam_cache,
		)
		.await?
	};

	Ok(data)
}
