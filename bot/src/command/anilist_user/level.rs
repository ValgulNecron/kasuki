use once_cell::sync::Lazy;
use sea_orm::EntityTrait;
use serenity::all::{CommandInteraction, Context as SerenityContext};

use crate::command::anilist_user::user::get_user;
use crate::command::command_trait::{Command, Embed, EmbedContent, EmbedType};
use crate::database::prelude::RegisteredUser;
use crate::database::registered_user::Column;
use crate::event_handler::BotData;
use crate::get_url;
use crate::helper::get_option::command::get_option_map_string;
use crate::structure::message::anilist_user::level::load_localization_level;
use crate::structure::run::anilist::user::{get_color, get_completed, get_user_url};
use anyhow::{Result, anyhow};
use sea_orm::ColumnTrait;
use sea_orm::QueryFilter;
use small_fixed_array::FixedString;

pub struct LevelCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl Command for LevelCommand {
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}
}

impl LevelCommand {
	pub async fn run_slash(self) -> Result<()> {
		let ctx = self.get_ctx();
		let bot_data = ctx.data::<BotData>().clone();
		let command_interaction = self.get_command_interaction();

		let anilist_cache = bot_data.anilist_cache.clone();
		let config = bot_data.config.clone();
		let db_config = config.db.clone();
		let map = get_option_map_string(command_interaction);

		let user = map.get(&FixedString::from_str_trunc("username"));

		let data = match user {
			Some(value) => get_user(value, anilist_cache).await?,
			None => {
				let user_id = &command_interaction.user.id.to_string();

				let connection = sea_orm::Database::connect(get_url(config.db.clone())).await?;

				let row = RegisteredUser::find()
					.filter(Column::UserId.eq(user_id))
					.one(&connection)
					.await?;

				let user = row.ok_or(anyhow!(
					"No user specified or linked to this discord account",
				))?;

				get_user(user.anilist_id.to_string().as_str(), anilist_cache).await?
			},
		};

		let user = data;

		let guild_id = match command_interaction.guild_id {
			Some(id) => id.to_string(),
			None => String::from("0"),
		};

		// Load the localized level strings
		let level_localised = load_localization_level(guild_id, db_config).await?;

		// Clone the manga and anime statistics
		let statistics = user.statistics.clone().unwrap();

		let manga = statistics.manga.clone();

		let anime = statistics.anime.clone();

		// Calculate the number of manga and anime completed
		let manga_completed = if let Some(manga) = manga.clone() {
			get_completed(manga.statuses.unwrap())
		} else {
			0
		};

		let anime_completed = if let Some(anime) = anime.clone() {
			get_completed(anime.statuses.unwrap())
		} else {
			0
		};

		// Get the number of chapters read and minutes watched
		let chap_read = if let Some(manga) = manga.clone() {
			manga.chapters_read
		} else {
			0
		};

		let tw = if let Some(anime) = anime.clone() {
			anime.minutes_watched
		} else {
			0
		};

		// Calculate the experience points
		let xp = (8.0 * (manga_completed + anime_completed) as f64)
			+ (2.0 * chap_read as f64)
			+ (tw as f64 * 0.5);

		// Get the username
		let username = user.name.clone();

		// Calculate the level and progress
		let (level, actual, next_xp): (u32, f64, f64) = get_level(xp);

		let mut content = EmbedContent::new(user.clone().name)
			.description(
				level_localised
					.desc
					.replace("$username$", username.as_str())
					.replace("$level$", level.to_string().as_str())
					.replace("$xp$", xp.to_string().as_str())
					.replace("$actual$", actual.to_string().as_str())
					.replace("$next$", next_xp.to_string().as_str()),
			)
			.thumbnail(Some(user.clone().avatar.unwrap().large.clone().unwrap()))
			.url(Some(get_user_url(&user.id)))
			.command_type(EmbedType::First)
			.colour(Some(get_color(user.clone())));

		// Add the user's banner image to the embed if it exists
		if let Some(banner_image) = &user.banner_image {
			content = content.images_url(Some(banner_image.clone()));
		}

		self.send_embed(vec![content]).await
	}
}

pub static LEVELS: Lazy<[(u32, f64, f64); 102]> = Lazy::new(|| {
	[
		(0, 0.0, xp_required_for_level(1)),
		(1, xp_required_for_level(1), xp_required_for_level(2)),
		(2, xp_required_for_level(2), xp_required_for_level(3)),
		(3, xp_required_for_level(3), xp_required_for_level(4)),
		(4, xp_required_for_level(4), xp_required_for_level(5)),
		(5, xp_required_for_level(5), xp_required_for_level(6)),
		(6, xp_required_for_level(6), xp_required_for_level(7)),
		(7, xp_required_for_level(7), xp_required_for_level(8)),
		(8, xp_required_for_level(8), xp_required_for_level(9)),
		(9, xp_required_for_level(9), xp_required_for_level(10)),
		(10, xp_required_for_level(10), xp_required_for_level(11)),
		(11, xp_required_for_level(11), xp_required_for_level(12)),
		(12, xp_required_for_level(12), xp_required_for_level(13)),
		(13, xp_required_for_level(13), xp_required_for_level(14)),
		(14, xp_required_for_level(14), xp_required_for_level(15)),
		(15, xp_required_for_level(15), xp_required_for_level(16)),
		(16, xp_required_for_level(16), xp_required_for_level(17)),
		(17, xp_required_for_level(17), xp_required_for_level(18)),
		(18, xp_required_for_level(18), xp_required_for_level(19)),
		(19, xp_required_for_level(19), xp_required_for_level(20)),
		(20, xp_required_for_level(20), xp_required_for_level(21)),
		(21, xp_required_for_level(21), xp_required_for_level(22)),
		(22, xp_required_for_level(22), xp_required_for_level(23)),
		(23, xp_required_for_level(23), xp_required_for_level(24)),
		(24, xp_required_for_level(24), xp_required_for_level(25)),
		(25, xp_required_for_level(25), xp_required_for_level(26)),
		(26, xp_required_for_level(26), xp_required_for_level(27)),
		(27, xp_required_for_level(27), xp_required_for_level(28)),
		(28, xp_required_for_level(28), xp_required_for_level(29)),
		(29, xp_required_for_level(29), xp_required_for_level(30)),
		(30, xp_required_for_level(30), xp_required_for_level(31)),
		(31, xp_required_for_level(31), xp_required_for_level(32)),
		(32, xp_required_for_level(32), xp_required_for_level(33)),
		(33, xp_required_for_level(33), xp_required_for_level(34)),
		(34, xp_required_for_level(34), xp_required_for_level(35)),
		(35, xp_required_for_level(35), xp_required_for_level(36)),
		(36, xp_required_for_level(36), xp_required_for_level(37)),
		(37, xp_required_for_level(37), xp_required_for_level(38)),
		(38, xp_required_for_level(38), xp_required_for_level(39)),
		(39, xp_required_for_level(39), xp_required_for_level(40)),
		(40, xp_required_for_level(40), xp_required_for_level(41)),
		(41, xp_required_for_level(41), xp_required_for_level(42)),
		(42, xp_required_for_level(42), xp_required_for_level(43)),
		(43, xp_required_for_level(43), xp_required_for_level(44)),
		(44, xp_required_for_level(44), xp_required_for_level(45)),
		(45, xp_required_for_level(45), xp_required_for_level(46)),
		(46, xp_required_for_level(46), xp_required_for_level(47)),
		(47, xp_required_for_level(47), xp_required_for_level(48)),
		(48, xp_required_for_level(48), xp_required_for_level(49)),
		(49, xp_required_for_level(49), xp_required_for_level(50)),
		(50, xp_required_for_level(50), xp_required_for_level(51)),
		(51, xp_required_for_level(51), xp_required_for_level(52)),
		(52, xp_required_for_level(52), xp_required_for_level(53)),
		(53, xp_required_for_level(53), xp_required_for_level(54)),
		(54, xp_required_for_level(54), xp_required_for_level(55)),
		(55, xp_required_for_level(55), xp_required_for_level(56)),
		(56, xp_required_for_level(56), xp_required_for_level(57)),
		(57, xp_required_for_level(57), xp_required_for_level(58)),
		(58, xp_required_for_level(58), xp_required_for_level(59)),
		(59, xp_required_for_level(59), xp_required_for_level(60)),
		(60, xp_required_for_level(60), xp_required_for_level(61)),
		(61, xp_required_for_level(61), xp_required_for_level(62)),
		(62, xp_required_for_level(62), xp_required_for_level(63)),
		(63, xp_required_for_level(63), xp_required_for_level(64)),
		(64, xp_required_for_level(64), xp_required_for_level(65)),
		(65, xp_required_for_level(65), xp_required_for_level(66)),
		(66, xp_required_for_level(66), xp_required_for_level(67)),
		(67, xp_required_for_level(67), xp_required_for_level(68)),
		(68, xp_required_for_level(68), xp_required_for_level(69)),
		(69, xp_required_for_level(69), xp_required_for_level(70)),
		(70, xp_required_for_level(70), xp_required_for_level(71)),
		(71, xp_required_for_level(71), xp_required_for_level(72)),
		(72, xp_required_for_level(72), xp_required_for_level(73)),
		(73, xp_required_for_level(73), xp_required_for_level(74)),
		(74, xp_required_for_level(74), xp_required_for_level(75)),
		(75, xp_required_for_level(75), xp_required_for_level(76)),
		(76, xp_required_for_level(76), xp_required_for_level(77)),
		(77, xp_required_for_level(77), xp_required_for_level(78)),
		(78, xp_required_for_level(78), xp_required_for_level(79)),
		(79, xp_required_for_level(79), xp_required_for_level(80)),
		(80, xp_required_for_level(80), xp_required_for_level(81)),
		(81, xp_required_for_level(81), xp_required_for_level(82)),
		(82, xp_required_for_level(82), xp_required_for_level(83)),
		(83, xp_required_for_level(83), xp_required_for_level(84)),
		(84, xp_required_for_level(84), xp_required_for_level(85)),
		(85, xp_required_for_level(85), xp_required_for_level(86)),
		(86, xp_required_for_level(86), xp_required_for_level(87)),
		(87, xp_required_for_level(87), xp_required_for_level(88)),
		(88, xp_required_for_level(88), xp_required_for_level(89)),
		(89, xp_required_for_level(89), xp_required_for_level(90)),
		(90, xp_required_for_level(90), xp_required_for_level(91)),
		(91, xp_required_for_level(91), xp_required_for_level(92)),
		(92, xp_required_for_level(92), xp_required_for_level(93)),
		(93, xp_required_for_level(93), xp_required_for_level(94)),
		(94, xp_required_for_level(94), xp_required_for_level(95)),
		(95, xp_required_for_level(95), xp_required_for_level(96)),
		(96, xp_required_for_level(96), xp_required_for_level(97)),
		(97, xp_required_for_level(97), xp_required_for_level(98)),
		(98, xp_required_for_level(98), xp_required_for_level(99)),
		(99, xp_required_for_level(99), xp_required_for_level(100)),
		(100, xp_required_for_level(100), f64::MAX),
		(101, f64::MAX, f64::MAX),
	]
});

fn get_level(xp: f64) -> (u32, f64, f64) {
	for &(level, required_xp, next_level_required_xp) in LEVELS.iter().rev() {
		if xp >= required_xp {
			let level_progress = xp - required_xp;

			let level_progress_total = next_level_required_xp - required_xp;

			return (level, level_progress, level_progress_total);
		}
	}

	(0, 0.0, 20.0)
}

fn xp_required_for_level(level: u32) -> f64 {
	match level {
		0..=9 => (level as f64).powf(3f64),
		10..=29 => (level as f64).powf(4f64),
		30..=39 => (level as f64).powf(5f64),
		40..=49 => (level as f64).powf(6f64),
		50..=59 => (level as f64).powf(7f64),
		60..=69 => (level as f64).powf(8f64),
		70..=79 => (level as f64).powf(9f64),
		80..=89 => (level as f64).powf(10f64),
		90..=100 => (level as f64).powf(11f64),
		_ => f64::MAX,
	}
}
