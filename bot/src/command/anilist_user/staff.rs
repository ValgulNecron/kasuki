use std::sync::Arc;

use crate::command::command_trait::{Command, Embed, EmbedContent, EmbedType, SlashCommand};
use crate::event_handler::BotData;
use crate::helper::convert_flavored_markdown::convert_anilist_flavored_to_discord_flavored_markdown;
use crate::helper::get_option::command::get_option_map_string;
use crate::helper::make_graphql_cached::make_request_anilist;
use crate::structure::message::anilist_user::staff::load_localization_staff;
use crate::structure::run::anilist::staff::{
	FuzzyDate, Staff, StaffQuerryId, StaffQuerryIdVariables, StaffQuerrySearch,
	StaffQuerrySearchVariables,
};
use anyhow::{anyhow, Result};
use cynic::{GraphQlResponse, QueryBuilder};
use moka::future::Cache;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use small_fixed_array::FixedString;
use tokio::sync::RwLock;

pub struct StaffCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl Command for StaffCommand {
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}
}

impl SlashCommand for StaffCommand {
	async fn run_slash(&self) -> Result<()> {
		let ctx = self.get_ctx();
		let bot_data = ctx.data::<BotData>().clone();
		let command_interaction = &self.command_interaction;

		let config = bot_data.config.clone();

		let anilist_cache = bot_data.anilist_cache.clone();
		let staff = get_staff(command_interaction, anilist_cache).await?;

		let va = staff
			.characters
			.unwrap()
			.nodes
			.unwrap()
			.iter()
			.filter_map(|x| {
				let x = x.clone().unwrap();
				let name = x.name.unwrap();
				let full = name.full.as_deref();
				let native = name.native.as_deref();
				get_full_name(full, native)
			})
			.take(5)
			.collect::<Vec<String>>()
			.join("\n");

		let media = staff
			.staff_media
			.unwrap()
			.edges
			.unwrap()
			.iter()
			.filter_map(|x| {
				let node = x.clone().unwrap().node.unwrap();
				let title = node.title.unwrap();
				let romaji = title.romaji.as_deref();
				let english = title.english.as_deref();
				get_full_name(romaji, english)
			})
			.take(5)
			.collect::<Vec<String>>()
			.join("\n");

		let job = staff.primary_occupations.unwrap()[0]
			.clone()
			.unwrap_or_default();

		let gender = staff.gender.clone().unwrap_or(String::from("Unknown."));

		let lang = staff.language_v2.unwrap_or_default();

		let guild_id = match command_interaction.guild_id {
			Some(id) => id.to_string(),
			None => String::from("0"),
		};
		let staff_localised = load_localization_staff(guild_id, config.db.clone()).await?;

		let mut fields = vec![
			(staff_localised.media, media, true),
			(staff_localised.occupation, job, true),
			(staff_localised.gender, gender, true),
			(staff_localised.lang, lang, true),
		];
		if !va.is_empty() {
			fields.push((staff_localised.va, va, true))
		}

		let age = staff.age;

		if age.is_some() {
			fields.push((
				staff_localised.age,
				age.unwrap_or_default().to_string(),
				true,
			))
		}

		let name = staff.name.unwrap();
		if staff.date_of_birth.is_some() {
			let date_of_birth = get_date(staff.date_of_birth.clone());
			if date_of_birth != String::new() {
				fields.push((staff_localised.date_of_birth, date_of_birth, true));
			}
		}

		if staff.date_of_death.is_some() {
			let date_of_death = get_date(staff.date_of_death.clone());
			if date_of_death != String::new() {
				fields.push((staff_localised.date_of_death, date_of_death, true));
			}
		}

		let name = name.full.unwrap_or(
			name.user_preferred
				.unwrap_or(name.native.unwrap_or(String::from("Unknown."))),
		);

		let embed_content = EmbedContent {
			title: name,
			description: convert_anilist_flavored_to_discord_flavored_markdown(
				staff.description.unwrap_or_default(),
			),
			thumbnail: staff.image.unwrap().large,
			url: staff.site_url,
			command_type: EmbedType::First,
			colour: None,
			fields: fields,
			images: None,
			action_row: None,
		};
		self.send_embed(embed_content).await
	}
}

async fn get_staff(
	command_interaction: &CommandInteraction, anilist_cache: Arc<RwLock<Cache<String, String>>>,
) -> Result<Staff> {
	let map = get_option_map_string(command_interaction);

	let value = map
		.get(&FixedString::from_str_trunc("staff_name"))
		.ok_or(anyhow!("No staff name specified"))?;

	let staff = if value.parse::<i32>().is_ok() {
		let var = StaffQuerryIdVariables {
			id: Some(value.parse()?),
		};

		let operation = StaffQuerryId::build(var);

		let data: GraphQlResponse<StaffQuerryId> =
			make_request_anilist(operation, false, anilist_cache).await?;

		data.data.unwrap().staff.unwrap()
	} else {
		let var = StaffQuerrySearchVariables {
			search: Some(value),
		};

		let operation = StaffQuerrySearch::build(var);

		let data: GraphQlResponse<StaffQuerrySearch> =
			make_request_anilist(operation, false, anilist_cache).await?;

		data.data.unwrap().staff.unwrap()
	};

	Ok(staff)
}

fn get_date(option: Option<FuzzyDate>) -> String {
	if option.is_none() {
		return String::new();
	}
	let date = option.unwrap();

	let mut date_string = String::new();

	let mut day = false;

	let mut month = false;

	if let Some(m) = date.month {
		month = true;

		date_string.push_str(m.to_string().as_str())
	}

	if let Some(d) = date.day {
		day = true;

		if month {
			date_string.push('/')
		}

		date_string.push_str(d.to_string().as_str())
	}

	if let Some(y) = date.year {
		if day {
			date_string.push('/')
		}

		date_string.push_str(y.to_string().as_str())
	}

	date_string
}

fn get_full_name(a: Option<&str>, b: Option<&str>) -> Option<String> {
	match (a, b) {
		(Some(a), Some(b)) => Some(format!("{}/{}", a, b)),
		(Some(a), None) => Some(a.to_string()),
		(None, Some(b)) => Some(b.to_string()),
		(None, None) => None,
	}
}
