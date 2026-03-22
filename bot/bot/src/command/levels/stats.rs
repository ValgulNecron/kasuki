use crate::command::context::CommandContext;
use crate::command::embed_content::{CommandFiles, EmbedContent, EmbedsContents};
use crate::constant::COLOR;
use crate::helper::progress_bar_generator::generate_progress_bar_image_in_memory;
use anyhow::{anyhow, Result};
use kasuki_macros::slash_command;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use sea_orm::{ColumnTrait, Condition};
use serenity::all::{ChannelId, CommandInteraction, Context as SerenityContext};
use serenity::model::Colour;
use shared::database::prelude::{Message as DatabaseMessage, Vocal as DatabaseVocal};
use shared::database::{message, vocal};
use shared::localization::{Loader, USABLE_LOCALES};
use uuid::Uuid;

#[slash_command(
	name = "stats", desc = "Get stats for level.",
	command_type = SubCommand(parent = "levels"),
	contexts = [Guild, PrivateChannel],
	install_contexts = [Guild],
)]
async fn levels_stats_command(self_: LevelsStatsCommand) -> Result<EmbedsContents<'_>> {
	let cx = CommandContext::new(
		self_.get_ctx().clone(),
		self_.get_command_interaction().clone(),
	);

	let channels_id = cx
		.command_interaction
		.guild_id
		.unwrap()
		.channels(&cx.ctx.http)
		.await?;
	let vec_channel_id: Vec<ChannelId> = channels_id.iter().map(|a| a.id).collect();
	let vec_string: Vec<String> = vec_channel_id.iter().map(|id| id.to_string()).collect();
	let user_id = cx.command_interaction.user.id.to_string();

	let condition = Condition::all()
		.add(message::Column::UserId.eq(user_id.clone()))
		.add(message::Column::ChannelId.is_in(vec_string.clone()));

	let messages = DatabaseMessage::find()
		.filter(condition)
		.all(&*cx.db)
		.await?;

	let total_message = messages.len() as i128;

	let condition = Condition::all()
		.add(vocal::Column::UserId.eq(user_id))
		.add(vocal::Column::ChannelId.is_in(vec_string));

	let vocals = DatabaseVocal::find().filter(condition).all(&*cx.db).await?;

	let total_vocal = vocals.len() as i128;

	let mut total_vocal_len: i128 = 0;
	for vocal in vocals {
		total_vocal_len += vocal.duration as i128;
	}

	let hours = total_vocal_len / 3600;
	let minutes = (total_vocal_len % 3600) / 60;
	let seconds = total_vocal_len % 60;

	let xp_message = total_message;
	let xp_vocal = total_vocal;
	let xp_vocal_len = total_vocal_len / 10;

	let xp = xp_vocal_len + xp_message + xp_vocal;
	let level = get_level(xp);

	let current_level_xp = get_xp_for_level(level);
	let next_level_xp = get_xp_for_next_level(level);
	let xp_progress = xp - current_level_xp;
	let xp_needed = next_level_xp - current_level_xp;

	let user_color = cx.command_interaction.user.accent_colour;
	let (progress_file, _) = create_progress_bar(xp_progress, xp_needed, user_color).await?;
	let progress_filename = format!("attachment://{}", progress_file.filename.clone());

	let lang_id = cx.lang_id().await;

	let next_level = level + 1;

	let level_progress_args = shared::fluent_args!(
		"current_level" => level,
		"next_level" => next_level,
		"current_xp" => xp_progress.to_string(),
		"next_level_xp" => xp_needed.to_string(),
	);

	let vocal_args = shared::fluent_args!("session" => total_vocal.to_string());

	let vocal_len_args = shared::fluent_args!(
		"hours" => hours.to_string(),
		"minutes" => minutes.to_string(),
		"seconds" => seconds.to_string(),
	);

	let message_args = shared::fluent_args!("message" => total_message.to_string());

	let xp_message_args = shared::fluent_args!("xp" => xp_message.to_string());

	let xp_vocal_args = shared::fluent_args!("xp" => xp_vocal.to_string());

	let xp_vocal_len_args = shared::fluent_args!("xp" => xp_vocal_len.to_string());

	let xp_total_args = shared::fluent_args!("xp" => xp.to_string());

	let title = USABLE_LOCALES.lookup(&lang_id, "levels_stats-title");
	let embed_content = EmbedContent::new(title.clone())
		.images_url(progress_filename)
		.fields(vec![
			(format!("Level {}", level), String::new(), false),
			(
				USABLE_LOCALES.lookup(&lang_id, "levels_stats-level_progress_title"),
				String::new(),
				false,
			),
			(
				USABLE_LOCALES.lookup_with_args(
					&lang_id,
					"levels_stats-level_progress",
					&level_progress_args,
				),
				String::new(),
				false,
			),
			(
				USABLE_LOCALES.lookup(&lang_id, "levels_stats-vocal_title"),
				String::new(),
				false,
			),
			(
				USABLE_LOCALES.lookup_with_args(&lang_id, "levels_stats-vocal", &vocal_args),
				String::new(),
				true,
			),
			(
				USABLE_LOCALES.lookup_with_args(
					&lang_id,
					"levels_stats-vocal_len",
					&vocal_len_args,
				),
				String::new(),
				true,
			),
			(
				USABLE_LOCALES.lookup(&lang_id, "levels_stats-message_title"),
				String::new(),
				false,
			),
			(
				USABLE_LOCALES.lookup_with_args(&lang_id, "levels_stats-message", &message_args),
				String::new(),
				true,
			),
			(
				USABLE_LOCALES.lookup(&lang_id, "levels_stats-xp_title"),
				String::new(),
				false,
			),
			(
				USABLE_LOCALES.lookup_with_args(
					&lang_id,
					"levels_stats-xp_message",
					&xp_message_args,
				),
				String::new(),
				true,
			),
			(
				USABLE_LOCALES.lookup_with_args(&lang_id, "levels_stats-xp_vocal", &xp_vocal_args),
				String::new(),
				true,
			),
			(
				USABLE_LOCALES.lookup_with_args(
					&lang_id,
					"levels_stats-xp_vocal_len",
					&xp_vocal_len_args,
				),
				String::new(),
				true,
			),
			(
				USABLE_LOCALES.lookup_with_args(&lang_id, "levels_stats-xp_total", &xp_total_args),
				String::new(),
				false,
			),
		]);

	let embed_contents = EmbedsContents::new(vec![embed_content]).add_files(vec![progress_file]);

	Ok(embed_contents)
}

fn get_level(xp: i128) -> i32 {
	match xp {
		0..=500 => 1,
		501..=1500 => 2,
		1501..=3000 => 3,
		3001..=5000 => 4,
		5001..=8000 => 5,
		8001..=12000 => 6,
		12001..=17000 => 7,
		17001..=23000 => 8,
		23001..=30000 => 9,
		30001..=38000 => 10,
		38001..=47000 => 11,
		47001..=57000 => 12,
		57001..=68000 => 13,
		68001..=80000 => 14,
		80001..=93000 => 15,
		93001..=107000 => 16,
		107001..=122000 => 17,
		122001..=138000 => 18,
		138001..=155000 => 19,
		_ => 20,
	}
}

fn get_xp_for_level(level: i32) -> i128 {
	match level {
		1 => 0,
		2 => 501,
		3 => 1501,
		4 => 3001,
		5 => 5001,
		6 => 8001,
		7 => 12001,
		8 => 17001,
		9 => 23001,
		10 => 30001,
		11 => 38001,
		12 => 47001,
		13 => 57001,
		14 => 68001,
		15 => 80001,
		16 => 93001,
		17 => 107001,
		18 => 122001,
		19 => 138001,
		20 => 155001,
		_ => 155001,
	}
}

fn get_xp_for_next_level(level: i32) -> i128 {
	match level {
		1 => 501,
		2 => 1501,
		3 => 3001,
		4 => 5001,
		5 => 8001,
		6 => 12001,
		7 => 17001,
		8 => 23001,
		9 => 30001,
		10 => 38001,
		11 => 47001,
		12 => 57001,
		13 => 68001,
		14 => 80001,
		15 => 93001,
		16 => 107001,
		17 => 122001,
		18 => 138001,
		19 => 155001,
		_ => 999999,
	}
}

async fn create_progress_bar(
	current: i128, max: i128, user_color: Option<Colour>,
) -> Result<(CommandFiles, i32)> {
	let percent = if max > 0 {
		((current as f64 / max as f64) * 100.0) as i32
	} else {
		100
	};

	let percent = percent.max(0).min(100);

	let color = user_color.unwrap_or(COLOR);
	let rgb_color = [color.r(), color.b(), color.g(), 255];

	let image_data = generate_progress_bar_image_in_memory(percent as u32, rgb_color)
		.map_err(|e| anyhow!("Failed to generate progress bar image: {}", e))?;

	let uuid = Uuid::new_v4();
	let filename = format!("progress_{}.png", uuid);

	let file = CommandFiles::new(filename.clone(), image_data);

	Ok((file, percent))
}
