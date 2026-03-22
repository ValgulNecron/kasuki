use std::sync::Arc;

use anyhow::Context;
use markdown_converter::vndb::convert_vndb_markdown;
use unic_langid::LanguageIdentifier;

use crate::cache::CacheInterface;
use crate::localization::{Loader, USABLE_LOCALES};
use crate::service::types::*;
use crate::vndb::character::{self, get_character};
use crate::vndb::game::{self, get_vn};
use crate::vndb::producer::{self, get_producer};
use crate::vndb::staff::{self, get_staff};
use crate::vndb::stats::{self, get_stats};
use crate::vndb::user::{get_user, VnUser};

/// Filter an image URL based on content safety ratings.
/// Returns `None` if the image is too sexual (> 1.5) or violent (> 1.0), or if there's no image.
fn filter_safe_image_url<I: VndbImage>(image: &Option<I>) -> Option<String> {
	match image {
		// VNDB rates 0-2; 1.5 sexual / 1.0 violence are safe-for-Discord thresholds
		// (violence has a stricter cap because graphic content is less tolerated)
		Some(img) if img.sexual() <= 1.5 && img.violence() <= 1.0 => Some(img.url().to_string()),
		_ => None,
	}
}

/// Trait to abstract over different VNDB image types (game::Image vs character::Image).
trait VndbImage {
	fn sexual(&self) -> f64;
	fn violence(&self) -> f64;
	fn url(&self) -> &str;
}

impl VndbImage for character::Image {
	fn sexual(&self) -> f64 {
		self.sexual
	}
	fn violence(&self) -> f64 {
		self.violence
	}
	fn url(&self) -> &str {
		&self.url
	}
}

impl VndbImage for game::Image {
	fn sexual(&self) -> f64 {
		self.sexual
	}
	fn violence(&self) -> f64 {
		self.violence
	}
	fn url(&self) -> &str {
		&self.url
	}
}

/// Collect up to 10 items into a comma-separated string, then push as a display field if non-empty.
fn push_list_field(
	fields: &mut Vec<DisplayField>, lang_id: &LanguageIdentifier, key: &str,
	items: impl Iterator<Item = String>,
) {
	let joined = items.take(10).collect::<Vec<_>>().join(", ");
	if !joined.is_empty() {
		fields.push((USABLE_LOCALES.lookup(lang_id, key), joined, true));
	}
}

/// Build display fields from a VNDB character.
fn build_character_fields(
	character: &character::Character, lang_id: &LanguageIdentifier,
) -> Vec<DisplayField> {
	let mut fields = vec![];

	if let Some(blood_type) = &character.blood_type {
		fields.push((
			USABLE_LOCALES.lookup(lang_id, "vn_character-blood_type"),
			blood_type.clone(),
			true,
		));
	}

	if let Some(height) = character.height {
		fields.push((
			USABLE_LOCALES.lookup(lang_id, "vn_character-height"),
			format!("{}cm", height),
			true,
		));
	}

	if let Some(weight) = character.weight {
		fields.push((
			USABLE_LOCALES.lookup(lang_id, "vn_character-weight"),
			format!("{}kg", weight),
			true,
		));
	}

	if let Some(age) = character.age {
		fields.push((
			USABLE_LOCALES.lookup(lang_id, "vn_character-age"),
			age.to_string(),
			true,
		));
	}

	if let Some(bust) = character.bust {
		fields.push((
			USABLE_LOCALES.lookup(lang_id, "vn_character-bust"),
			format!("{}cm", bust),
			true,
		));
	}

	if let Some(waist) = character.waist {
		fields.push((
			USABLE_LOCALES.lookup(lang_id, "vn_character-waist"),
			format!("{}cm", waist),
			true,
		));
	}

	if let Some(hips) = character.hips {
		fields.push((
			USABLE_LOCALES.lookup(lang_id, "vn_character-hip"),
			format!("{}cm", hips),
			true,
		));
	}

	if let Some(cup) = &character.cup {
		fields.push((
			USABLE_LOCALES.lookup(lang_id, "vn_character-cup"),
			cup.clone(),
			true,
		));
	}

	// sex[0] = apparent gender, sex[1] = spoiler/true gender; wrap second in Discord spoiler tags
	let sex = format!("{}, ||{}||", character.sex[0], character.sex[1]);
	fields.push((
		USABLE_LOCALES.lookup(lang_id, "vn_character-sex"),
		sex,
		true,
	));

	if let Some(birthday) = &character.birthday {
		fields.push((
			USABLE_LOCALES.lookup(lang_id, "vn_character-birthday"),
			format!("{:02}/{:02}", birthday[0], birthday[1]),
			true,
		));
	}

	// Cap at 10 to stay within Discord embed field value limits (1024 chars)
	push_list_field(
		&mut fields,
		lang_id,
		"vn_character-vns",
		character.vns.iter().map(|vn| vn.title.clone()),
	);

	push_list_field(
		&mut fields,
		lang_id,
		"vn_character-traits",
		character.traits.iter().map(|t| t.name.clone()),
	);

	fields
}

/// Build a CharacterResult from a VNDB character (pure transformation).
fn build_character_result(
	character: &character::Character, lang_id: &LanguageIdentifier,
) -> CharacterResult {
	let fields = build_character_fields(character, lang_id);
	let description = character
		.description
		.as_deref()
		.map(|d| convert_vndb_markdown(d).to_string());
	let image_url = filter_safe_image_url(&character.image);

	CharacterResult {
		name: character.name.clone(),
		id: character.id.clone(),
		description,
		fields,
		image_url,
		url: format!("https://vndb.org/{}", character.id),
	}
}

/// Build display fields from a VNDB game.
fn build_game_fields(vn: &game::VN, lang_id: &LanguageIdentifier) -> Vec<DisplayField> {
	let mut fields = vec![];

	if let Some(released) = &vn.released {
		fields.push((
			USABLE_LOCALES.lookup(lang_id, "vn_game-released"),
			released.clone(),
			true,
		));
	}

	// All collection fields capped at 10 to fit Discord embed field limits (1024 chars)
	push_list_field(
		&mut fields,
		lang_id,
		"vn_game-platforms",
		vn.platforms.iter().cloned(),
	);

	if let Some(playtime) = vn.length_minutes {
		fields.push((
			USABLE_LOCALES.lookup(lang_id, "vn_game-playtime"),
			playtime.to_string(),
			true,
		));
	}

	push_list_field(
		&mut fields,
		lang_id,
		"vn_game-tags",
		vn.tags.iter().map(|tag| tag.name.clone()),
	);
	push_list_field(
		&mut fields,
		lang_id,
		"vn_game-developers",
		vn.developers.iter().map(|dev| dev.name.clone()),
	);
	push_list_field(
		&mut fields,
		lang_id,
		"vn_game-staff",
		vn.staff.iter().map(|s| s.name.clone()),
	);
	push_list_field(
		&mut fields,
		lang_id,
		"vn_game-characters",
		vn.va.iter().map(|va| va.character.name.clone()),
	);

	fields
}

/// Build a GameResult from a VNDB game (pure transformation).
fn build_game_result(vn: &game::VN, lang_id: &LanguageIdentifier) -> GameResult {
	let fields = build_game_fields(vn, lang_id);
	let description = vn
		.description
		.as_deref()
		.map(|d| convert_vndb_markdown(d).to_string());
	let image_url = filter_safe_image_url(&vn.image);

	GameResult {
		title: vn.title.clone(),
		id: vn.id.clone(),
		description,
		fields,
		image_url,
		url: format!("https://vndb.org/{}", vn.id),
	}
}

/// Build a StatsResult from VNDB stats (pure transformation).
fn build_stats_result(stats: &stats::Stats, lang_id: &LanguageIdentifier) -> StatsResult {
	let fields = vec![
		(
			USABLE_LOCALES.lookup(lang_id, "vn_stats-chars"),
			stats.chars.to_string(),
			true,
		),
		(
			USABLE_LOCALES.lookup(lang_id, "vn_stats-producer"),
			stats.producers.to_string(),
			true,
		),
		(
			USABLE_LOCALES.lookup(lang_id, "vn_stats-release"),
			stats.releases.to_string(),
			true,
		),
		(
			USABLE_LOCALES.lookup(lang_id, "vn_stats-staff"),
			stats.staff.to_string(),
			true,
		),
		(
			USABLE_LOCALES.lookup(lang_id, "vn_stats-tags"),
			stats.tags.to_string(),
			true,
		),
		(
			USABLE_LOCALES.lookup(lang_id, "vn_stats-traits"),
			stats.traits.to_string(),
			true,
		),
		(
			USABLE_LOCALES.lookup(lang_id, "vn_stats-vns"),
			stats.vn.to_string(),
			true,
		),
		(
			USABLE_LOCALES.lookup(lang_id, "vn_stats-api"),
			String::from("VNDB API"),
			true,
		),
	];

	let title = USABLE_LOCALES.lookup(lang_id, "vn_stats-title");

	StatsResult { title, fields }
}

/// Build a UserResult from a VNDB user (pure transformation).
fn build_user_result(user: &VnUser, lang_id: &LanguageIdentifier) -> UserResult {
	let fields = vec![
		(
			USABLE_LOCALES.lookup(lang_id, "vn_user-id"),
			user.id.clone(),
			true,
		),
		(
			USABLE_LOCALES.lookup(lang_id, "vn_user-playtime"),
			user.lengthvotes.to_string(),
			true,
		),
		(
			USABLE_LOCALES.lookup(lang_id, "vn_user-playtimesum"),
			user.lengthvotes_sum.to_string(),
			true,
		),
		(
			USABLE_LOCALES.lookup(lang_id, "vn_user-name"),
			user.username.clone(),
			true,
		),
	];

	let title_args = crate::fluent_args!("user" => user.username.clone());
	let title = USABLE_LOCALES.lookup_with_args(lang_id, "vn_user-title", &title_args);

	UserResult { title, fields }
}

/// Build a StaffResult from a VNDB staff member (pure transformation).
fn build_staff_result(s: &staff::Staff, lang_id: &LanguageIdentifier) -> StaffResult {
	let fields = vec![
		(
			USABLE_LOCALES.lookup(lang_id, "vn_staff-lang"),
			s.lang.clone(),
			true,
		),
		(
			USABLE_LOCALES.lookup(lang_id, "vn_staff-aid"),
			s.aid.to_string(),
			true,
		),
		(
			USABLE_LOCALES.lookup(lang_id, "vn_staff-gender"),
			s.gender.clone().unwrap_or_default(),
			true,
		),
		(
			USABLE_LOCALES.lookup(lang_id, "vn_staff-main"),
			s.ismain.to_string(),
			true,
		),
	];

	let description = s.description.as_deref().map(|d| convert_vndb_markdown(d).to_string());

	StaffResult {
		name: s.name.clone(),
		id: s.id.clone(),
		description,
		fields,
		url: format!("https://vndb.org/{}", s.id),
	}
}

/// Build a ProducerResult from a VNDB producer (pure transformation).
fn build_producer_result(p: &producer::Producer, lang_id: &LanguageIdentifier) -> ProducerResult {
	let mut fields = vec![];

	if let Some(lang) = &p.lang {
		fields.push((
			USABLE_LOCALES.lookup(lang_id, "vn_producer-lang"),
			lang.clone(),
			true,
		));
	}

	if let Some(aliases) = &p.aliases {
		push_list_field(
			&mut fields,
			lang_id,
			"vn_producer-aliases",
			aliases.iter().cloned(),
		);
	}

	if let Some(results_type) = &p.results_type {
		fields.push((
			USABLE_LOCALES.lookup(lang_id, "vn_producer-prod_type"),
			results_type.to_string(),
			true,
		));
	}

	let description = p
		.description
		.as_deref()
		.map(|d| convert_vndb_markdown(d).to_string());

	ProducerResult {
		name: p.name.clone(),
		id: p.id.clone(),
		description,
		fields,
		url: format!("https://vndb.org/{}", p.id),
	}
}

/// Fetch a VNDB character and format it for display.
pub async fn lookup_character(
	name: String, lang_id: &LanguageIdentifier, vndb_cache: Arc<CacheInterface>,
	http_client: &reqwest::Client,
) -> anyhow::Result<CharacterResult> {
	let result = get_character(name.clone(), vndb_cache, http_client)
		.await
		.context(format!("Failed to get character information for: {}", name))?;

	let character = result.results.first().context("No character found")?;

	Ok(build_character_result(character, lang_id))
}

/// Fetch a VN game and format it for display.
pub async fn lookup_game(
	title: String, lang_id: &LanguageIdentifier, vndb_cache: Arc<CacheInterface>,
	http_client: &reqwest::Client,
) -> anyhow::Result<GameResult> {
	let vn_root = get_vn(title, vndb_cache, http_client).await?;
	let vn = vn_root.results.first().context("No VN found")?;

	Ok(build_game_result(vn, lang_id))
}

/// Fetch VNDB statistics and format for display.
pub async fn lookup_stats(
	lang_id: &LanguageIdentifier, vndb_cache: Arc<CacheInterface>, http_client: &reqwest::Client,
) -> anyhow::Result<StatsResult> {
	let stats = get_stats(vndb_cache, http_client).await?;

	Ok(build_stats_result(&stats, lang_id))
}

/// Fetch a VNDB user and format for display.
pub async fn lookup_user(
	username: &str, lang_id: &LanguageIdentifier, vndb_cache: Arc<CacheInterface>,
	http_client: &reqwest::Client,
) -> anyhow::Result<UserResult> {
	let path = format!("/user?q={}&fields=lengthvotes,lengthvotes_sum", username);
	let user = get_user(path, vndb_cache, http_client).await?;

	Ok(build_user_result(&user, lang_id))
}

/// Fetch a VN staff member and format for display.
pub async fn lookup_staff(
	name: String, lang_id: &LanguageIdentifier, vndb_cache: Arc<CacheInterface>,
	http_client: &reqwest::Client,
) -> anyhow::Result<StaffResult> {
	let result = get_staff(name, vndb_cache, http_client).await?;
	let s = result.results.first().context("No staff found")?;

	Ok(build_staff_result(s, lang_id))
}

/// Fetch a VN producer and format for display.
pub async fn lookup_producer(
	name: String, lang_id: &LanguageIdentifier, vndb_cache: Arc<CacheInterface>,
	http_client: &reqwest::Client,
) -> anyhow::Result<ProducerResult> {
	let result = get_producer(name, vndb_cache, http_client).await?;
	let p = result.results.first().context("No producer found")?;

	Ok(build_producer_result(p, lang_id))
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::str::FromStr;

	fn en_us() -> LanguageIdentifier {
		LanguageIdentifier::from_str("en-US").unwrap()
	}

	#[test]
	fn safe_character_image_is_returned() {
		let image = Some(character::Image {
			sexual: 0.0,
			violence: 0.0,
			url: "https://example.com/safe.jpg".to_string(),
		});
		assert_eq!(
			filter_safe_image_url(&image),
			Some("https://example.com/safe.jpg".to_string())
		);
	}

	#[test]
	fn borderline_safe_character_image_is_returned() {
		let image = Some(character::Image {
			sexual: 1.5,
			violence: 1.0,
			url: "https://example.com/borderline.jpg".to_string(),
		});
		assert_eq!(
			filter_safe_image_url(&image),
			Some("https://example.com/borderline.jpg".to_string())
		);
	}

	#[test]
	fn nsfw_character_image_is_filtered() {
		let image = Some(character::Image {
			sexual: 2.0,
			violence: 0.0,
			url: "https://example.com/nsfw.jpg".to_string(),
		});
		assert_eq!(filter_safe_image_url(&image), None);
	}

	#[test]
	fn violent_character_image_is_filtered() {
		let image = Some(character::Image {
			sexual: 0.0,
			violence: 1.5,
			url: "https://example.com/violent.jpg".to_string(),
		});
		assert_eq!(filter_safe_image_url(&image), None);
	}

	#[test]
	fn no_image_returns_none() {
		let image: Option<character::Image> = None;
		assert_eq!(filter_safe_image_url(&image), None);
	}

	#[test]
	fn safe_game_image_is_returned() {
		let image = Some(game::Image {
			sexual: 0.5,
			violence: 0.5,
			url: "https://example.com/game.jpg".to_string(),
		});
		assert_eq!(
			filter_safe_image_url(&image),
			Some("https://example.com/game.jpg".to_string())
		);
	}

	#[test]
	fn nsfw_game_image_is_filtered() {
		let image = Some(game::Image {
			sexual: 1.6,
			violence: 0.0,
			url: "https://example.com/nsfw_game.jpg".to_string(),
		});
		assert_eq!(filter_safe_image_url(&image), None);
	}

	fn make_character(
		height: Option<i64>, weight: Option<i64>, age: Option<i64>,
	) -> character::Character {
		character::Character {
			blood_type: Some("O".to_string()),
			description: Some("A brave warrior.".to_string()),
			traits: vec![
				character::Trait {
					spoiler: 0,
					name: "Brave".to_string(),
					id: "t1".to_string(),
				},
				character::Trait {
					spoiler: 0,
					name: "Strong".to_string(),
					id: "t2".to_string(),
				},
			],
			waist: Some(60),
			name: "Saber".to_string(),
			height,
			cup: Some("B".to_string()),
			sex: vec!["Female".to_string(), "Female".to_string()],
			vns: vec![character::VN {
				id: "v1".to_string(),
				title: "Fate/stay night".to_string(),
			}],
			image: Some(character::Image {
				sexual: 0.5,
				violence: 0.0,
				url: "https://example.com/saber.jpg".to_string(),
			}),
			hips: Some(85),
			id: "c1".to_string(),
			bust: Some(73),
			weight,
			age,
			birthday: Some(vec![6, 24]),
		}
	}

	#[test]
	fn character_fields_include_all_present_attributes() {
		let character = make_character(Some(154), Some(42), Some(25));
		let lang_id = en_us();
		let fields = build_character_fields(&character, &lang_id);

		// Should have: blood_type, height, weight, age, bust, waist, hips, cup, sex, birthday, vns, traits = 12
		assert_eq!(fields.len(), 12);

		// Check specific field values
		assert!(fields.iter().any(|(_, v, _)| v == "154cm"));
		assert!(fields.iter().any(|(_, v, _)| v == "42kg"));
		assert!(fields.iter().any(|(_, v, _)| v == "25"));
		assert!(fields.iter().any(|(_, v, _)| v == "73cm")); // bust
		assert!(fields.iter().any(|(_, v, _)| v == "60cm")); // waist
		assert!(fields.iter().any(|(_, v, _)| v == "85cm")); // hips
		assert!(fields.iter().any(|(_, v, _)| v == "B")); // cup
		assert!(fields.iter().any(|(_, v, _)| v == "O")); // blood type
		assert!(fields.iter().any(|(_, v, _)| v == "06/24")); // birthday
		assert!(fields.iter().any(|(_, v, _)| v == "Female, ||Female||")); // sex
		assert!(fields.iter().any(|(_, v, _)| v == "Fate/stay night")); // vns
		assert!(fields.iter().any(|(_, v, _)| v == "Brave, Strong")); // traits
	}

	#[test]
	fn character_fields_skip_none_attributes() {
		let character = make_character(None, None, None);
		let lang_id = en_us();
		let fields = build_character_fields(&character, &lang_id);

		// Without height, weight, age: blood_type, bust, waist, hips, cup, sex, birthday, vns, traits = 9
		assert_eq!(fields.len(), 9);
		assert!(!fields
			.iter()
			.any(|(_, v, _)| v.contains("cm") && v.contains("154")));
	}

	#[test]
	fn character_result_has_correct_url_and_image() {
		let character = make_character(Some(154), Some(42), None);
		let lang_id = en_us();
		let result = build_character_result(&character, &lang_id);

		assert_eq!(result.name, "Saber");
		assert_eq!(result.id, "c1");
		assert_eq!(result.url, "https://vndb.org/c1");
		assert_eq!(
			result.image_url,
			Some("https://example.com/saber.jpg".to_string())
		);
		assert!(result.description.is_some());
	}

	#[test]
	fn character_result_filters_nsfw_image() {
		let mut character = make_character(None, None, None);
		character.image = Some(character::Image {
			sexual: 2.0,
			violence: 0.0,
			url: "https://example.com/nsfw.jpg".to_string(),
		});
		let result = build_character_result(&character, &en_us());
		assert_eq!(result.image_url, None);
	}

	#[test]
	fn character_result_no_image() {
		let mut character = make_character(None, None, None);
		character.image = None;
		let result = build_character_result(&character, &en_us());
		assert_eq!(result.image_url, None);
	}

	#[test]
	fn character_vns_capped_at_10() {
		let mut character = make_character(None, None, None);
		character.vns = (0..20)
			.map(|i| character::VN {
				id: format!("v{}", i),
				title: format!("VN {}", i),
			})
			.collect();
		let fields = build_character_fields(&character, &en_us());
		let vns_field = fields.iter().find(|(_, v, _)| v.contains("VN ")).unwrap();
		// Should only contain 10 entries
		assert_eq!(vns_field.1.matches(", ").count(), 9); // 10 items = 9 commas
	}

	#[test]
	fn character_traits_capped_at_10() {
		let mut character = make_character(None, None, None);
		character.traits = (0..20)
			.map(|i| character::Trait {
				spoiler: 0,
				name: format!("Trait{}", i),
				id: format!("t{}", i),
			})
			.collect();
		let fields = build_character_fields(&character, &en_us());
		let traits_field = fields.iter().find(|(_, v, _)| v.contains("Trait")).unwrap();
		assert_eq!(traits_field.1.matches(", ").count(), 9);
	}

	fn make_vn() -> game::VN {
		game::VN {
			olang: "ja".to_string(),
			va: vec![game::Va {
				character: game::Character {
					id: "c1".to_string(),
					name: "Saber".to_string(),
				},
			}],
			released: Some("2004-01-30".to_string()),
			id: "v11".to_string(),
			image: Some(game::Image {
				sexual: 0.0,
				violence: 0.0,
				url: "https://example.com/fsn.jpg".to_string(),
			}),
			staff: vec![game::Staff {
				id: "s1".to_string(),
				name: "Nasu Kinoko".to_string(),
				role: "scenario".to_string(),
			}],
			rating: Some(8.5),
			length_minutes: Some(3000.0),
			platforms: vec!["win".to_string(), "ps2".to_string()],
			title: "Fate/stay night".to_string(),
			average: Some(8.2),
			titles: vec![],
			votecount: 1000.0,
			languages: vec!["ja".to_string(), "en".to_string()],
			aliases: vec![],
			tags: vec![
				game::Tags {
					name: "Fantasy".to_string(),
					id: "g1".to_string(),
					rating: 3.0,
					spoiler: 0.0,
				},
				game::Tags {
					name: "Action".to_string(),
					id: "g2".to_string(),
					rating: 2.5,
					spoiler: 0.0,
				},
			],
			description: Some("A visual novel about the Holy Grail War.".to_string()),
			devstatus: game::DevStatus::Finished,
			developers: vec![game::Developers {
				name: "TYPE-MOON".to_string(),
				id: "p1".to_string(),
			}],
		}
	}

	#[test]
	fn game_fields_include_all_data() {
		let vn = make_vn();
		let lang_id = en_us();
		let fields = build_game_fields(&vn, &lang_id);

		// released, platforms, playtime, tags, developers, staff, characters = 7
		assert_eq!(fields.len(), 7);

		assert!(fields.iter().any(|(_, v, _)| v == "2004-01-30"));
		assert!(fields.iter().any(|(_, v, _)| v == "win, ps2"));
		assert!(fields.iter().any(|(_, v, _)| v == "3000")); // playtime
		assert!(fields.iter().any(|(_, v, _)| v == "Fantasy, Action"));
		assert!(fields.iter().any(|(_, v, _)| v == "TYPE-MOON"));
		assert!(fields.iter().any(|(_, v, _)| v == "Nasu Kinoko"));
		assert!(fields.iter().any(|(_, v, _)| v == "Saber"));
	}

	#[test]
	fn game_fields_skip_empty_collections() {
		let mut vn = make_vn();
		vn.released = None;
		vn.platforms = vec![];
		vn.length_minutes = None;
		vn.tags = vec![];
		vn.developers = vec![];
		vn.staff = vec![];
		vn.va = vec![];

		let fields = build_game_fields(&vn, &en_us());
		assert_eq!(fields.len(), 0);
	}

	#[test]
	fn game_result_has_correct_url_and_image() {
		let vn = make_vn();
		let result = build_game_result(&vn, &en_us());

		assert_eq!(result.title, "Fate/stay night");
		assert_eq!(result.id, "v11");
		assert_eq!(result.url, "https://vndb.org/v11");
		assert_eq!(
			result.image_url,
			Some("https://example.com/fsn.jpg".to_string())
		);
		assert!(result.description.is_some());
	}

	#[test]
	fn game_result_filters_nsfw_image() {
		let mut vn = make_vn();
		vn.image = Some(game::Image {
			sexual: 2.0,
			violence: 2.0,
			url: "https://example.com/nsfw.jpg".to_string(),
		});
		let result = build_game_result(&vn, &en_us());
		assert_eq!(result.image_url, None);
	}

	#[test]
	fn game_result_no_description() {
		let mut vn = make_vn();
		vn.description = None;
		let result = build_game_result(&vn, &en_us());
		assert_eq!(result.description, None);
	}

	#[test]
	fn stats_result_has_8_fields() {
		let stats = stats::Stats {
			chars: 100,
			producers: 50,
			releases: 200,
			staff: 75,
			tags: 300,
			traits: 400,
			vn: 150,
		};
		let result = build_stats_result(&stats, &en_us());
		assert_eq!(result.fields.len(), 8);
		assert!(!result.title.is_empty());
	}

	#[test]
	fn stats_result_contains_correct_values() {
		let stats = stats::Stats {
			chars: 12345,
			producers: 678,
			releases: 9999,
			staff: 111,
			tags: 222,
			traits: 333,
			vn: 444,
		};
		let result = build_stats_result(&stats, &en_us());

		assert!(result.fields.iter().any(|(_, v, _)| v == "12345"));
		assert!(result.fields.iter().any(|(_, v, _)| v == "678"));
		assert!(result.fields.iter().any(|(_, v, _)| v == "9999"));
		assert!(result.fields.iter().any(|(_, v, _)| v == "111"));
		assert!(result.fields.iter().any(|(_, v, _)| v == "222"));
		assert!(result.fields.iter().any(|(_, v, _)| v == "333"));
		assert!(result.fields.iter().any(|(_, v, _)| v == "444"));
		assert!(result.fields.iter().any(|(_, v, _)| v == "VNDB API"));
	}

	#[test]
	fn stats_all_fields_are_inline() {
		let stats = stats::Stats {
			chars: 1,
			producers: 1,
			releases: 1,
			staff: 1,
			tags: 1,
			traits: 1,
			vn: 1,
		};
		let result = build_stats_result(&stats, &en_us());
		assert!(result.fields.iter().all(|(_, _, inline)| *inline));
	}

	#[test]
	fn user_result_has_4_fields() {
		let user = VnUser {
			id: "u12345".to_string(),
			lengthvotes: 42,
			lengthvotes_sum: 5000,
			username: "testuser".to_string(),
		};
		let result = build_user_result(&user, &en_us());

		assert_eq!(result.fields.len(), 4);
		assert!(result.fields.iter().any(|(_, v, _)| v == "u12345"));
		assert!(result.fields.iter().any(|(_, v, _)| v == "42"));
		assert!(result.fields.iter().any(|(_, v, _)| v == "5000"));
		assert!(result.fields.iter().any(|(_, v, _)| v == "testuser"));
	}

	#[test]
	fn user_result_title_contains_username() {
		let user = VnUser {
			id: "u1".to_string(),
			lengthvotes: 0,
			lengthvotes_sum: 0,
			username: "MyUser".to_string(),
		};
		let result = build_user_result(&user, &en_us());
		assert!(result.title.contains("MyUser"));
	}

	#[test]
	fn staff_result_has_correct_structure() {
		let s = staff::Staff {
			ismain: true,
			aid: 42,
			name: "Nasu Kinoko".to_string(),
			gender: Some("m".to_string()),
			lang: "ja".to_string(),
			description: Some("A famous scenario writer.".to_string()),
			id: "s1".to_string(),
		};
		let result = build_staff_result(&s, &en_us());

		assert_eq!(result.name, "Nasu Kinoko");
		assert_eq!(result.id, "s1");
		assert_eq!(result.url, "https://vndb.org/s1");
		assert_eq!(result.fields.len(), 4);
		assert!(result.description.is_some());

		assert!(result.fields.iter().any(|(_, v, _)| v == "ja"));
		assert!(result.fields.iter().any(|(_, v, _)| v == "42"));
		assert!(result.fields.iter().any(|(_, v, _)| v == "m"));
		assert!(result.fields.iter().any(|(_, v, _)| v == "true"));
	}

	#[test]
	fn staff_result_missing_gender_defaults_to_empty() {
		let s = staff::Staff {
			ismain: false,
			aid: 0,
			name: "Unknown".to_string(),
			gender: None,
			lang: "en".to_string(),
			description: None,
			id: "s99".to_string(),
		};
		let result = build_staff_result(&s, &en_us());

		let gender_field = result.fields.iter().find(|(_, v, _)| v.is_empty()).unwrap();
		assert_eq!(gender_field.1, "");
	}

	#[test]
	fn producer_result_all_fields_present() {
		let p = producer::Producer {
			results_type: Some(producer::Type::Company),
			lang: Some("ja".to_string()),
			name: "TYPE-MOON".to_string(),
			description: Some("A famous visual novel studio.".to_string()),
			aliases: Some(vec!["TM".to_string(), "Type Moon".to_string()]),
			id: "p1".to_string(),
		};
		let result = build_producer_result(&p, &en_us());

		assert_eq!(result.name, "TYPE-MOON");
		assert_eq!(result.id, "p1");
		assert_eq!(result.url, "https://vndb.org/p1");
		assert_eq!(result.fields.len(), 3); // lang, aliases, type
		assert!(result.description.is_some());

		assert!(result.fields.iter().any(|(_, v, _)| v == "ja"));
		assert!(result.fields.iter().any(|(_, v, _)| v == "TM, Type Moon"));
		assert!(result.fields.iter().any(|(_, v, _)| v == "Company"));
	}

	#[test]
	fn producer_result_no_optional_fields() {
		let p = producer::Producer {
			results_type: None,
			lang: None,
			name: "Indie Dev".to_string(),
			description: None,
			aliases: None,
			id: "p99".to_string(),
		};
		let result = build_producer_result(&p, &en_us());

		assert_eq!(result.fields.len(), 0);
		assert_eq!(result.description, None);
		assert_eq!(result.url, "https://vndb.org/p99");
	}

	#[test]
	fn producer_aliases_capped_at_10() {
		let p = producer::Producer {
			results_type: None,
			lang: None,
			name: "Test".to_string(),
			description: None,
			aliases: Some((0..20).map(|i| format!("Alias{}", i)).collect()),
			id: "p1".to_string(),
		};
		let result = build_producer_result(&p, &en_us());
		let aliases_field = result
			.fields
			.iter()
			.find(|(_, v, _)| v.contains("Alias"))
			.unwrap();
		assert_eq!(aliases_field.1.matches(", ").count(), 9); // 10 items = 9 commas
	}

	#[test]
	fn producer_type_individual() {
		let p = producer::Producer {
			results_type: Some(producer::Type::Individual),
			lang: None,
			name: "Solo Dev".to_string(),
			description: None,
			aliases: None,
			id: "p2".to_string(),
		};
		let result = build_producer_result(&p, &en_us());
		assert!(result.fields.iter().any(|(_, v, _)| v == "Individual"));
	}

	#[test]
	fn producer_type_amateur_group() {
		let p = producer::Producer {
			results_type: Some(producer::Type::AmateurGroup),
			lang: None,
			name: "Circle".to_string(),
			description: None,
			aliases: None,
			id: "p3".to_string(),
		};
		let result = build_producer_result(&p, &en_us());
		assert!(result.fields.iter().any(|(_, v, _)| v == "Amateur Group"));
	}
}
