use anyhow::{anyhow, Result};
use std::io::{Cursor, Read};
use std::sync::Arc;

use crate::command::context::CommandContext;
use crate::command::embed_content::{EmbedContent, EmbedsContents};
use crate::helper::get_option::subcommand_group::get_option_map_string_subcommand_group;
use crate::helper::trimer::trim_webhook;
use crate::structure::run::anilist::minimal_anime::{
	Media, MediaTitle, MinimalAnimeId, MinimalAnimeIdVariables, MinimalAnimeSearch,
	MinimalAnimeSearchVariables,
};
use base64::engine::general_purpose::STANDARD;
use base64::read::DecoderReader;
use base64::Engine as _;
use bytes::Bytes;
use chrono::Utc;
use cynic::{GraphQlResponse, QueryBuilder};
use fluent_templates::fluent_bundle::FluentValue;
use fluent_templates::Loader;
use image::imageops::FilterType;
use image::{guess_format, GenericImageView, ImageFormat};
use kasuki_macros::slash_command;
use reqwest::get;
use sea_orm::ActiveValue::Set;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use sea_orm::{ColumnTrait, DatabaseConnection};
use serde_json::json;
use serenity::all::{
	ChannelId, CommandInteraction, Context as SerenityContext, CreateAttachment, EditWebhook,
	GenericChannelId,
};
use shared::anilist::make_request::make_request_anilist;
use shared::cache::CacheInterface;
use shared::database::activity_data;
use shared::database::activity_data::Column;
use shared::database::prelude::ActivityData;
use shared::localization::USABLE_LOCALES;
use std::borrow::Cow;
use std::collections::HashMap;
use tracing::trace;

#[slash_command(
	name = "add_anime_activity", desc = "Add an anime activity.",
	command_type = SubCommandGroup(parent = "admin", group = "anilist"),
	args = [
		(name = "anime_name", desc = "Name of the anime you want to add as an activity.", arg_type = String, required = true, autocomplete = true),
		(name = "delays", desc = "A delay in seconds.", arg_type = Integer, required = false, autocomplete = false)
	],
)]
async fn add_activity_command(self_: AddActivityCommand) -> Result<EmbedsContents<'_>> {
	let cx = CommandContext::new(
		self_.get_ctx().clone(),
		self_.get_command_interaction().clone(),
	);

	let map = get_option_map_string_subcommand_group(&cx.command_interaction);
	let anime = map.get("anime_name").cloned().unwrap_or(String::new());

	let media = get_minimal_anime_media(anime.to_string(), cx.anilist_cache.clone()).await?;

	let lang_id = cx.lang_id().await;

	let anime_id = media.id;
	let url = format!("https://anilist.co/anime/{}", anime_id);
	let exist = check_if_activity_exist(anime_id, cx.guild_id.clone(), cx.db.clone()).await;

	let title = media.title.ok_or(anyhow!("No title for the media"))?;
	let anime_name = get_name(title);

	let mut args = HashMap::new();
	args.insert(
		Cow::Borrowed("anime"),
		FluentValue::from(anime_name.as_str()),
	);

	if exist {
		let embed_content =
			EmbedContent::new(USABLE_LOCALES.lookup(&lang_id, "admin_anilist_add_activity-fail"))
				.description(USABLE_LOCALES.lookup_with_args(
					&lang_id,
					"admin_anilist_add_activity-fail_desc",
					&args,
				))
				.url(url);

		let embed_contents = EmbedsContents::new(vec![embed_content]);

		return Ok(embed_contents);
	}

	let channel_id = cx.command_interaction.channel_id;

	let delay = map
		.get("delay")
		.unwrap_or(&String::from("0"))
		.parse()
		.unwrap_or(0);

	let trimmed_anime_name = if anime_name.len() >= 50 {
		trim_webhook(anime_name.clone(), 50 - anime_name.len() as i32)
	} else {
		anime_name.clone()
	};

	let image_url = media.cover_image.ok_or(
		anyhow!("No cover image for this media"),
	)?.extra_large.
		unwrap_or(
			"https://imgs.search.brave.com/CYnhSvdQcm9aZe3wG84YY0B19zT2wlAuAkiAGu0mcLc/rs:fit:640:400:1/g:ce/aHR0cDovL3d3dy5m/cmVtb250Z3VyZHdh/cmEub3JnL3dwLWNv/bnRlbnQvdXBsb2Fk/cy8yMDIwLzA2L25v/LWltYWdlLWljb24t/Mi5wbmc"
				.to_string()
		);
	let bytes = get(image_url.clone()).await?.bytes().await?;
	let buf = resize_image(&bytes).await?;
	let base64 = STANDARD.encode(buf.into_inner());
	let image = format!("data:image/jpeg;base64,{}", base64);

	let next_airing = media.next_airing_episode.clone().ok_or(anyhow!(
		"No next episode found for {} on anilist",
		anime_name
	))?;
	let timestamp = next_airing.airing_at as i64;
	let chrono = chrono::DateTime::<Utc>::from_timestamp(timestamp, 0)
		.unwrap_or_default()
		.naive_utc();

	let webhook = get_webhook(
		&cx.ctx,
		channel_id,
		image.clone(),
		base64.clone(),
		trimmed_anime_name.clone(),
	)
	.await?;

	ActivityData::insert(activity_data::ActiveModel {
		anime_id: Set(media.id),
		timestamp: Set(chrono),
		server_id: Set(cx.guild_id.clone()),
		webhook: Set(Some(webhook)),
		episode: Set(next_airing.episode),
		name: Set(trimmed_anime_name),
		delay: Set(delay),
		image: Set(image.clone()),
		channel_id: Set(None),
	})
	.exec(&*cx.db)
	.await?;

	let embed_content =
		EmbedContent::new(USABLE_LOCALES.lookup(&lang_id, "admin_anilist_add_activity-success"))
			.description(USABLE_LOCALES.lookup_with_args(
				&lang_id,
				"admin_anilist_add_activity-success_desc",
				&args,
			))
			.url(url);

	let embed_contents = EmbedsContents::new(vec![embed_content]);

	Ok(embed_contents)
}

async fn resize_image(image_bytes: &Bytes) -> Result<Cursor<Vec<u8>>> {
	let image_bytes = image_bytes.clone();
	tokio::task::spawn_blocking(move || {
		let image = image::load_from_memory_with_format(&image_bytes, guess_format(&image_bytes)?)?;

		let (width, height) = image.dimensions();

		let (crop_x, crop_y, square_size) = calculate_crop_params(width, height);

		let resized_image = image
			.crop_imm(crop_x, crop_y, square_size, square_size)
			.resize_exact(128, 128, FilterType::Nearest);

		let mut buffer = Cursor::new(Vec::new());

		resized_image.write_to(&mut buffer, ImageFormat::Jpeg)?;

		Ok(buffer)
	})
	.await?
}

fn calculate_crop_params(width: u32, height: u32) -> (u32, u32, u32) {
	let square_size = width.min(height);

	let crop_x = (width - square_size) / 2;

	let crop_y = (height - square_size) / 2;

	(crop_x, crop_y, square_size)
}

async fn check_if_activity_exist(
	anime_id: i32, server_id: String, db_connection: Arc<DatabaseConnection>,
) -> bool {
	let row = match ActivityData::find()
		.filter(Column::ServerId.eq(server_id))
		.filter(Column::AnimeId.eq(anime_id))
		.one(&*db_connection)
		.await
	{
		Ok(row) => row,
		Err(_) => return false,
	};

	trace!(?row);

	row.is_some()
}

pub fn get_name(title: MediaTitle) -> String {
	let english_title = title.english;

	let romaji_title = title.romaji;

	let title = match (romaji_title, english_title) {
		(Some(romaji), Some(english)) => format!("{} / {}", english, romaji),
		(Some(romaji), None) => romaji,
		(None, Some(english)) => english,
		(None, None) => String::new(),
	};

	trace!(?title);

	title
}

async fn get_webhook(
	ctx: &SerenityContext, channel_id: GenericChannelId, image: String, base64: String,
	anime_name: String,
) -> Result<String> {
	trace!(?image);

	trace!(?anime_name);

	let webhook_info = json!({
		"avatar": image,
		"name": anime_name
	});

	let bot_id = ctx
		.http
		.get_current_application_info()
		.await?
		.id
		.to_string();

	trace!(?bot_id);

	let mut webhook_url = String::new();

	let webhooks = ctx
		.http
		.get_channel_webhooks(ChannelId::new(channel_id.get()))
		.await?;

	if webhooks.is_empty() {
		let webhook = ctx
			.http
			.create_webhook(ChannelId::new(channel_id.get()), &webhook_info, None)
			.await?;

		webhook_url = webhook.url()?;
	} else {
		for webhook in webhooks {
			if webhook
				.user
				.clone()
				.ok_or(anyhow!("webhook user not found"))?
				.id
				.to_string() == bot_id
			{
				webhook_url = webhook.url()?;

				break;
			}
		}

		if webhook_url.is_empty() {
			let webhook = ctx
				.http
				.create_webhook(ChannelId::new(channel_id.get()), &webhook_info, None)
				.await?;

			webhook_url = webhook.url()?;
		}
	}

	trace!(?webhook_url);

	let cursor = Cursor::new(base64);

	let mut decoder = DecoderReader::new(cursor, &STANDARD);

	let mut decoded_bytes = Vec::new();

	decoder.read_to_end(&mut decoded_bytes)?;

	let mut webhook = ctx.http.get_webhook_from_url(webhook_url.as_str()).await?;

	let attachment = CreateAttachment::bytes(decoded_bytes, "avatar");
	let attachment = attachment.encode("image/png").await?;
	let edit_webhook = EditWebhook::new().name(anime_name).avatar(attachment);

	webhook.edit(&ctx.http, edit_webhook).await?;

	Ok(webhook_url)
}

pub async fn get_minimal_anime_by_id(id: i32, cache: Arc<CacheInterface>) -> Result<Media> {
	trace!(?id);

	let query = MinimalAnimeIdVariables { id: Some(id) };

	let operation = MinimalAnimeId::build(query);

	let response: GraphQlResponse<MinimalAnimeId> =
		make_request_anilist(operation, true, cache).await?;

	let media = response
		.data
		.ok_or(anyhow!("Error with request"))?
		.media
		.ok_or(anyhow!("No media found"))?;

	Ok(media)
}

async fn get_minimal_anime_by_search(query: &str, cache: Arc<CacheInterface>) -> Result<Media> {
	trace!(?query);

	let search_query = MinimalAnimeSearchVariables {
		search: Some(query),
	};

	let operation = MinimalAnimeSearch::build(search_query);

	let response: GraphQlResponse<MinimalAnimeSearch> =
		make_request_anilist(operation, true, cache).await?;

	let media = response
		.data
		.ok_or(anyhow!("Error with request"))?
		.media
		.ok_or(anyhow!("No media found"))?;

	Ok(media)
}

pub async fn get_minimal_anime_media(anime: String, cache: Arc<CacheInterface>) -> Result<Media> {
	let media = if let Ok(id) = anime.parse::<i32>() {
		get_minimal_anime_by_id(id, cache).await?
	} else {
		get_minimal_anime_by_search(&anime, cache).await?
	};

	trace!(?media);

	Ok(media)
}
