use std::sync::Arc;

use crate::command::command_trait::{Command, Embed, SlashCommand};
use crate::event_handler::BotData;
use crate::helper::get_option::command::get_option_map_string;
use crate::helper::make_graphql_cached::make_request_anilist;
use crate::structure::run::anilist::character;
use crate::structure::run::anilist::character::{
	Character, CharacterQuerryId, CharacterQuerryIdVariables, CharacterQuerrySearch,
	CharacterQuerrySearchVariables,
};
use anyhow::{anyhow, Result};
use cynic::{GraphQlResponse, QueryBuilder};
use moka::future::Cache;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use small_fixed_array::FixedString;
use tokio::sync::RwLock;

pub struct CharacterCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl Command for CharacterCommand {
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}
}

impl SlashCommand for CharacterCommand {
	async fn run_slash(&self) -> Result<()> {
		let ctx = self.get_ctx();
		let bot_data = ctx.data::<BotData>().clone();
		let command_interaction = self.get_command_interaction();

		let anilist_cache = bot_data.anilist_cache.clone();
		let config = bot_data.config.clone();

		let map = get_option_map_string(command_interaction);
		let value = map
			.get(&FixedString::from_str_trunc("name"))
			.cloned()
			.unwrap_or(String::new());

		let data: Character = if value.parse::<i32>().is_ok() {
			get_character_by_id(value.parse::<i32>().unwrap(), anilist_cache).await?
		} else {
			let var = CharacterQuerrySearchVariables {
				search: Some(&*value),
			};

			let operation = CharacterQuerrySearch::build(var);

			let data: GraphQlResponse<CharacterQuerrySearch> =
				make_request_anilist(operation, false, anilist_cache).await?;

			data.data.unwrap().character.unwrap()
		};

		let content =
			character::character_content(command_interaction, data, config.db.clone()).await?;

		self.send_embed(content).await
	}
}

pub async fn get_character_by_id(
	value: i32, anilist_cache: Arc<RwLock<Cache<String, String>>>,
) -> Result<Character> {
	let var = CharacterQuerryIdVariables { id: Some(value) };

	let operation = CharacterQuerryId::build(var);

	let data: GraphQlResponse<CharacterQuerryId> =
		make_request_anilist(operation, false, anilist_cache).await?;

	Ok(match data.data {
		Some(data) => match data.character {
			Some(media) => media,
			None => return Err(anyhow!("No character found")),
		},
		None => return Err(anyhow!("No data found")),
	})
}
