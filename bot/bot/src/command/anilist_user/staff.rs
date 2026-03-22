use std::sync::Arc;

use crate::command::context::CommandContext;
use crate::command::embed_content::{EmbedContent, EmbedsContents};
use crate::helper::convert_flavored_markdown::convert_anilist_flavored_to_discord_flavored_markdown;
use crate::helper::get_option::command::get_option_map_string;
use crate::structure::run::anilist::staff::{
	Staff, StaffQuerryId, StaffQuerryIdVariables, StaffQuerrySearch, StaffQuerrySearchVariables,
};
use shared::anilist::character::{format_fuzzy_date, FuzzyDate as SharedFuzzyDate};
use anyhow::{anyhow, Result};
use cynic::{GraphQlResponse, QueryBuilder};
use fluent_templates::Loader;
use kasuki_macros::slash_command;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use shared::anilist::make_request::make_request_anilist;
use shared::cache::CacheInterface;
use shared::localization::USABLE_LOCALES;
use small_fixed_array::FixedString;

#[slash_command(
	name = "staff", desc = "Info of a staff.", command_type = ChatInput,
	contexts = [Guild, BotDm, PrivateChannel],
	install_contexts = [Guild, User],
	args = [(name = "staff_name", desc = "Name of the staff you want to check.", arg_type = String, required = true, autocomplete = true)],
)]
async fn staff_command(self_: StaffCommand) -> Result<EmbedsContents<'_>> {
	let cx = CommandContext::new(
		self_.get_ctx().clone(),
		self_.get_command_interaction().clone(),
	);
	let anilist_cache = cx.anilist_cache.clone();
	let staff = get_staff(&cx.command_interaction, anilist_cache).await?;

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

	let gender = staff.gender.unwrap_or_else(|| String::from("Unknown."));

	let lang = staff.language_v2.unwrap_or_default();

	let lang_id = cx.lang_id().await;

	let mut fields = vec![
		(
			USABLE_LOCALES.lookup(&lang_id, "anilist_user_staff-media"),
			media,
			true,
		),
		(
			USABLE_LOCALES.lookup(&lang_id, "anilist_user_staff-occupation"),
			job,
			true,
		),
		(
			USABLE_LOCALES.lookup(&lang_id, "anilist_user_staff-gender"),
			gender,
			true,
		),
		(
			USABLE_LOCALES.lookup(&lang_id, "anilist_user_staff-lang"),
			lang,
			true,
		),
	];

	if let Some(home_town) = staff.home_town {
		fields.push((
			USABLE_LOCALES.lookup(&lang_id, "anilist_user_staff-hometown"),
			home_town,
			true,
		))
	}

	if !va.is_empty() {
		fields.push((
			USABLE_LOCALES.lookup(&lang_id, "anilist_user_staff-va"),
			va,
			true,
		))
	}

	let age = staff.age;

	if age.is_some() {
		fields.push((
			USABLE_LOCALES.lookup(&lang_id, "anilist_user_staff-age"),
			age.unwrap_or_default().to_string(),
			true,
		))
	}

	let name = staff.name.unwrap();
	if let Some(ref dob) = staff.date_of_birth {
		let shared_dob = SharedFuzzyDate {
			month: dob.month,
			year: dob.year,
			day: dob.day,
		};
		let date_of_birth = format_fuzzy_date(&shared_dob);
		if !date_of_birth.is_empty() {
			fields.push((
				USABLE_LOCALES.lookup(&lang_id, "anilist_user_staff-date_of_birth"),
				date_of_birth,
				true,
			));
		}
	}

	if let Some(ref dod) = staff.date_of_death {
		let shared_dod = SharedFuzzyDate {
			month: dod.month,
			year: dod.year,
			day: dod.day,
		};
		let date_of_death = format_fuzzy_date(&shared_dod);
		if !date_of_death.is_empty() {
			fields.push((
				USABLE_LOCALES.lookup(&lang_id, "anilist_user_staff-date_of_death"),
				date_of_death,
				true,
			));
		}
	}

	let name = name.full.unwrap_or(
		name.user_preferred
			.unwrap_or(name.native.unwrap_or(String::from("Unknown."))),
	);

	let embed_content = EmbedContent::new(name)
		.description(convert_anilist_flavored_to_discord_flavored_markdown(
			staff.description.unwrap_or_default(),
		))
		.thumbnail(staff.image.unwrap().large.unwrap_or_default())
		.url(staff.site_url.unwrap_or_default())
		.fields(fields);

	let embed_contents = EmbedsContents::new(vec![embed_content]);

	Ok(embed_contents)
}

async fn get_staff(
	command_interaction: &CommandInteraction, anilist_cache: Arc<CacheInterface>,
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
			make_request_anilist(operation, true, anilist_cache).await?;

		data.data.unwrap().staff.unwrap()
	} else {
		let var = StaffQuerrySearchVariables {
			search: Some(value),
		};

		let operation = StaffQuerrySearch::build(var);

		let data: GraphQlResponse<StaffQuerrySearch> =
			make_request_anilist(operation, true, anilist_cache).await?;

		data.data.unwrap().staff.unwrap()
	};

	Ok(staff)
}

fn get_full_name(a: Option<&str>, b: Option<&str>) -> Option<String> {
	match (a, b) {
		(Some(a), Some(b)) => Some(format!("{}/{}", a, b)),
		(Some(a), None) => Some(a.to_string()),
		(None, Some(b)) => Some(b.to_string()),
		(None, None) => None,
	}
}
