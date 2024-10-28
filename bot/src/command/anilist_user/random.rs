use std::sync::Arc;

use cynic::{GraphQlResponse, QueryBuilder};
use moka::future::Cache;
use rand::{thread_rng, Rng};
use serenity::all::CreateInteractionResponse::Defer;
use serenity::all::{
	CommandInteraction, Context as SerenityContext, CreateInteractionResponseFollowup,
	CreateInteractionResponseMessage,
};
use small_fixed_array::FixedString;
use tokio::sync::RwLock;
use tracing::trace;

use crate::background_task::update_random_stats::update_random_stats;
use crate::command::command_trait::{Command, SlashCommand};
use crate::config::Config;
use crate::event_handler::BotData;
use crate::helper::convert_flavored_markdown::convert_anilist_flavored_to_discord_flavored_markdown;
use crate::helper::create_default_embed::get_default_embed;
use crate::helper::get_option::command::get_option_map_string;
use crate::helper::make_graphql_cached::make_request_anilist;
use crate::helper::trimer::trim;
use crate::structure::message::anilist_user::random::{load_localization_random, RandomLocalised};
use crate::structure::run::anilist::random::{
	Media, MediaType, RandomPageMedia, RandomPageMediaVariables,
};
use anyhow::{anyhow, Result};

pub struct RandomCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl Command for RandomCommand {
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}
}

impl SlashCommand for RandomCommand {
	async fn run_slash(&self) -> Result<()> {
		let ctx = self.get_ctx();
		let bot_data = ctx.data::<BotData>().clone();
		send_embed(
			&self.ctx,
			&self.command_interaction,
			bot_data.config.clone(),
			bot_data.anilist_cache.clone(),
		)
		.await
	}
}

async fn send_embed(
	ctx: &SerenityContext, command_interaction: &CommandInteraction, config: Arc<Config>,
	anilist_cache: Arc<RwLock<Cache<String, String>>>,
) -> Result<()> {
	// Retrieve the guild ID from the command interaction
	let guild_id = match command_interaction.guild_id {
		Some(id) => id.to_string(),
		None => String::from("0"),
	};

	// Load the localized random strings
	let random_localised = load_localization_random(guild_id, config.db.clone()).await?;

	// Retrieve the type of media (anime or manga) from the command interaction
	let map = get_option_map_string(command_interaction);

	let random_type = map
		.get(&FixedString::from_str_trunc("type"))
		.ok_or(anyhow!("No type specified"))?;

	// Create a deferred response to the command interaction
	let builder_message = Defer(CreateInteractionResponseMessage::new());

	// Send the deferred response
	command_interaction
		.create_response(&ctx.http, builder_message)
		.await?;

	let random_stats = update_random_stats(anilist_cache.clone()).await?;

	let last_page = if random_type.as_str() == "anime" {
		random_stats.anime_last_page
	} else if random_type.as_str() == "manga" {
		random_stats.manga_last_page
	} else {
		0
	};

	trace!(last_page);

	embed(
		last_page,
		random_type.to_string(),
		ctx,
		command_interaction,
		random_localised,
		anilist_cache,
	)
	.await?;

	Ok(())
}

async fn embed(
	last_page: i32, random_type: String, ctx: &SerenityContext,
	command_interaction: &CommandInteraction, random_localised: RandomLocalised,
	anilist_cache: Arc<RwLock<Cache<String, String>>>,
) -> Result<()> {
	let number = thread_rng().gen_range(1..=last_page);

	let mut var = RandomPageMediaVariables {
		media_type: None,
		page: Some(number),
	};

	if random_type == "manga" {
		var.media_type = Some(MediaType::Manga)
	} else {
		var.media_type = Some(MediaType::Anime);
	}

	let operation = RandomPageMedia::build(var);

	let data: Result<GraphQlResponse<RandomPageMedia>> =
		make_request_anilist(operation, false, anilist_cache).await;

	let data = data?;

	let data = data.data.unwrap();

	let inside_media = data.page.unwrap().media.unwrap()[0].clone().unwrap();

	let id = inside_media.id;

	let url = if random_type == "manga" {
		format!("https://anilist.co/manga/{}", id)
	} else {
		format!("https://anilist.co/anime/{}", id)
	};

	follow_up_message(
		ctx,
		command_interaction,
		inside_media,
		url,
		random_localised,
	)
	.await?;

	Ok(())
}

async fn follow_up_message(
	ctx: &SerenityContext, command_interaction: &CommandInteraction, media: Media, url: String,
	random_localised: RandomLocalised,
) -> Result<()> {
	let format = media.format.unwrap();

	let genres = media
		.genres
		.unwrap()
		.into_iter()
		.map(|genre| genre.unwrap().clone())
		.collect::<Vec<String>>()
		.join("/");

	let tags = media
		.tags
		.unwrap()
		.into_iter()
		.map(|tag| tag.unwrap().name.clone())
		.collect::<Vec<String>>()
		.join("/");

	let mut desc = media.description.unwrap();

	desc = convert_anilist_flavored_to_discord_flavored_markdown(desc);

	let length_diff = 4096 - desc.len() as i32;

	if length_diff <= 0 {
		desc = trim(desc.clone(), length_diff);
	}

	let title = media.title.clone().unwrap();

	let rj = title.native.unwrap_or_default();

	let user_pref = title.user_preferred.unwrap_or_default();

	let title = format!("{}/{}", user_pref, rj);

	let full_desc = random_localised
		.desc
		.replace("$format$", format.to_string().as_str())
		.replace("$tags$", tags.as_str())
		.replace("$genres$", genres.as_str())
		.replace("$desc$", desc.as_str());

	let builder_embed = get_default_embed(None)
		.title(title)
		.description(full_desc)
		.url(url);

	let builder_message = CreateInteractionResponseFollowup::new().embed(builder_embed);

	command_interaction
		.create_followup(&ctx.http, builder_message)
		.await?;

	Ok(())
}
