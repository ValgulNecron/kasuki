use crate::command::admin::anilist::add_activity::{get_minimal_anime_media, get_name};
use crate::command::command_trait::{Command, Embed, EmbedContent, EmbedType, SlashCommand};
use crate::config::DbConfig;
use crate::database::prelude::ActivityData;
use crate::event_handler::BotData;
use crate::get_url;
use crate::helper::get_option::subcommand_group::get_option_map_string_subcommand_group;
use crate::structure::message::admin::anilist::delete_activity::load_localization_delete_activity;
use anyhow::{Result, anyhow};
use sea_orm::ColumnTrait;
use sea_orm::{EntityTrait, ModelTrait, QueryFilter};
use serenity::all::{CommandInteraction, Context as SerenityContext};

pub struct DeleteActivityCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl Command for DeleteActivityCommand {
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}
}

impl SlashCommand for DeleteActivityCommand {
	async fn run_slash(&self) -> Result<()> {
		let ctx = self.get_ctx();
		let bot_data = ctx.data::<BotData>().clone();
		let anilist_cache = bot_data.anilist_cache.clone();

		let command_interaction = self.command_interaction.clone();

		let config = bot_data.config.clone();

		let map = get_option_map_string_subcommand_group(&command_interaction);

		let anime = map
			.get(&String::from("anime_name"))
			.cloned()
			.unwrap_or(String::new());

		let guild_id = match command_interaction.guild_id {
			Some(id) => id.to_string(),
			None => String::from("1"),
		};

		self.defer().await?;

		let delete_activity_localised_text =
			load_localization_delete_activity(guild_id.clone(), config.db.clone()).await?;

		let media = get_minimal_anime_media(anime.to_string(), anilist_cache).await?;

		let anime_id = media.id;

		remove_activity(guild_id.as_str(), &anime_id, config.db.clone()).await?;

		let title = media
			.title
			.ok_or(anyhow!(format!("Anime with id {} not found", anime_id)))?;

		let anime_name = get_name(title);

		let url = format!("https://anilist.co/anime/{}", media.id);

		let embed_content = EmbedContent {
			title: delete_activity_localised_text.success.clone(),
			description: delete_activity_localised_text
				.success_desc
				.replace("$anime$", anime_name.as_str()),
			thumbnail: None,
			url: Some(url),
			command_type: EmbedType::Followup,
			colour: None,
			fields: vec![],
			images: None,
			action_row: None,
			images_url: None,
		};

		self.send_embed(embed_content).await?;

		Ok(())
	}
}

async fn remove_activity(guild_id: &str, anime_id: &i32, db_config: DbConfig) -> Result<()> {
	let connection = sea_orm::Database::connect(get_url(db_config.clone())).await?;

	let activity = ActivityData::find()
		.filter(crate::database::activity_data::Column::ServerId.eq(guild_id))
		.filter(crate::database::activity_data::Column::AnimeId.eq(anime_id.to_string()))
		.one(&connection)
		.await?
		.ok_or(anyhow!(format!("Anime with id {} not found", anime_id)))?;

	activity.delete(&connection).await?;

	Ok(())
}
