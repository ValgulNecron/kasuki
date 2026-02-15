use crate::command::command::CommandRun;
use crate::command::embed_content::{CommandFiles, CommandType, EmbedContent, EmbedsContents};
use crate::constant::COLOR;
use crate::event_handler::BotData;
use crate::helper::progress_bar_generator::generate_progress_bar_image_in_memory;
use anyhow::{anyhow, Result};
use fluent_templates::fluent_bundle::FluentValue;
use kasuki_macros::slash_command;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use sea_orm::{ColumnTrait, Condition};
use serenity::all::{ChannelId, CommandInteraction, Context as SerenityContext};
use serenity::model::Colour;
use shared::database::prelude::{Message as DatabaseMessage, Vocal as DatabaseVocal};
use shared::database::{message, vocal};
use shared::localization::{get_language_identifier, Loader, USABLE_LOCALES};
use std::borrow::Cow;
use std::collections::HashMap;
use tracing::{debug, info};
use uuid::Uuid;

#[slash_command(
	name = "stats", desc = "Get stats for level.",
	command_type = SubCommand(parent = "levels"),
	contexts = [Guild, PrivateChannel],
	install_contexts = [Guild],
)]
async fn levels_stats_command(self_: LevelsStatsCommand) -> Result<EmbedsContents<'_>> {
	info!("Processing levels stats command");
	let _ = self_.defer().await;
	debug!("Command deferred");

	let ctx = self_.get_ctx();
	let bot_data = ctx.data::<BotData>().clone();
	let command_interaction = self_.get_command_interaction();
	debug!("Retrieved bot data and command interaction");

	debug!("Fetching channels for guild");
	let channels_id = command_interaction
		.guild_id
		.unwrap()
		.channels(&ctx.http)
		.await?;
	let vec_channel_id: Vec<ChannelId> = channels_id.iter().map(|a| a.id).collect();
	let vec_string: Vec<String> = vec_channel_id.iter().map(|id| id.to_string()).collect();
	let user_id = command_interaction.user.id.to_string();
	debug!(
		"User ID: {}, Channel count: {}",
		user_id,
		vec_channel_id.len()
	);

	debug!("Creating message query condition");
	let condition = Condition::all()
		.add(message::Column::UserId.eq(user_id.clone()))
		.add(message::Column::ChannelId.is_in(vec_string.clone()));

	let db_connection = bot_data.db_connection.clone();

	debug!("Querying database for user messages");
	let messages = DatabaseMessage::find()
		.filter(condition)
		.all(&*db_connection)
		.await?;

	let total_message = messages.len() as i128;
	debug!("Found {} messages for user", total_message);

	debug!("Creating vocal query condition");
	let condition = Condition::all()
		.add(vocal::Column::UserId.eq(user_id))
		.add(vocal::Column::ChannelId.is_in(vec_string));

	debug!("Querying database for user vocal sessions");
	let vocals = DatabaseVocal::find()
		.filter(condition)
		.all(&*db_connection)
		.await?;

	let total_vocal = vocals.len() as i128;
	debug!("Found {} vocal sessions for user", total_vocal);

	let mut total_vocal_len: i128 = 0;
	for vocal in vocals {
		total_vocal_len += vocal.duration as i128;
	}
	debug!("Total vocal duration: {} seconds", total_vocal_len);

	// Convert seconds to hours, minutes, seconds
	let hours = total_vocal_len / 3600;
	let minutes = (total_vocal_len % 3600) / 60;
	let seconds = total_vocal_len % 60;
	debug!("Formatted vocal time: {}h {}m {}s", hours, minutes, seconds);

	// Calculate XP components
	debug!("Calculating XP components");
	let xp_message = total_message;
	let xp_vocal = total_vocal;
	let xp_vocal_len = total_vocal_len / 10;
	debug!(
		"XP from messages: {}, XP from vocal sessions: {}, XP from vocal duration: {}",
		xp_message, xp_vocal, xp_vocal_len
	);

	// Calculate total XP and level
	let xp = xp_vocal_len + xp_message + xp_vocal;
	let level = get_level(xp);
	debug!("Total XP: {}, Current level: {}", xp, level);

	// Calculate level progression
	debug!("Calculating level progression");
	let current_level_xp = get_xp_for_level(level);
	let next_level_xp = get_xp_for_next_level(level);
	let xp_progress = xp - current_level_xp;
	let xp_needed = next_level_xp - current_level_xp;
	debug!("XP progress: {}/{} for next level", xp_progress, xp_needed);

	// Create progress bar with user color
	debug!("Creating progress bar with user color");
	let user_color = command_interaction.user.accent_colour;
	let (progress_file, percent) = create_progress_bar(xp_progress, xp_needed, user_color).await?;
	let progress_filename = format!("attachment://{}", progress_file.filename.clone());
	debug!(
		"Progress bar created with filename: {}, percent: {}",
		progress_file.filename, percent
	);

	debug!("Loading localization for levels stats");
	let lang_id = get_language_identifier(
		command_interaction.guild_id.unwrap().to_string(),
		db_connection,
	)
	.await;
	debug!("Localization loaded successfully");

	let next_level = level + 1;
	debug!("Current level: {}, Next level: {}", level, next_level);

	// Build arguments for level progress
	let mut level_progress_args: HashMap<Cow<'static, str>, FluentValue> = HashMap::new();
	level_progress_args.insert(Cow::Borrowed("current_level"), FluentValue::from(level));
	level_progress_args.insert(Cow::Borrowed("next_level"), FluentValue::from(next_level));
	level_progress_args.insert(
		Cow::Borrowed("current_xp"),
		FluentValue::from(xp_progress.to_string()),
	);
	level_progress_args.insert(
		Cow::Borrowed("next_level_xp"),
		FluentValue::from(xp_needed.to_string()),
	);

	// Build arguments for vocal
	let mut vocal_args: HashMap<Cow<'static, str>, FluentValue> = HashMap::new();
	vocal_args.insert(
		Cow::Borrowed("session"),
		FluentValue::from(total_vocal.to_string()),
	);

	// Build arguments for vocal_len
	let mut vocal_len_args: HashMap<Cow<'static, str>, FluentValue> = HashMap::new();
	vocal_len_args.insert(Cow::Borrowed("hours"), FluentValue::from(hours.to_string()));
	vocal_len_args.insert(
		Cow::Borrowed("minutes"),
		FluentValue::from(minutes.to_string()),
	);
	vocal_len_args.insert(
		Cow::Borrowed("seconds"),
		FluentValue::from(seconds.to_string()),
	);

	// Build arguments for message
	let mut message_args: HashMap<Cow<'static, str>, FluentValue> = HashMap::new();
	message_args.insert(
		Cow::Borrowed("message"),
		FluentValue::from(total_message.to_string()),
	);

	// Build arguments for XP values
	let mut xp_message_args: HashMap<Cow<'static, str>, FluentValue> = HashMap::new();
	xp_message_args.insert(
		Cow::Borrowed("xp"),
		FluentValue::from(xp_message.to_string()),
	);

	let mut xp_vocal_args: HashMap<Cow<'static, str>, FluentValue> = HashMap::new();
	xp_vocal_args.insert(Cow::Borrowed("xp"), FluentValue::from(xp_vocal.to_string()));

	let mut xp_vocal_len_args: HashMap<Cow<'static, str>, FluentValue> = HashMap::new();
	xp_vocal_len_args.insert(
		Cow::Borrowed("xp"),
		FluentValue::from(xp_vocal_len.to_string()),
	);

	let mut xp_total_args: HashMap<Cow<'static, str>, FluentValue> = HashMap::new();
	xp_total_args.insert(Cow::Borrowed("xp"), FluentValue::from(xp.to_string()));

	// Create embed with title and fields
	debug!("Creating embed content with fields");
	let title = USABLE_LOCALES.lookup(&lang_id, "levels_stats-title");
	let embed_content = EmbedContent::new(title.clone())
		.images_url(progress_filename)
		.fields(vec![
			// Level Information
			(format!("Level {}", level), String::new(), false),
			// Level Progression Section
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
			// Voice Activity Section
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
			// Message Activity Section
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
			// XP Breakdown Section
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
	debug!("Embed content created with title: {}", title);

	debug!("Creating final embed contents with CommandType::Followup");
	let mut embed_contents = EmbedsContents::new(CommandType::Followup, vec![embed_content]);
	embed_contents.add_files(vec![progress_file]);
	debug!("Added progress bar file to embed contents");

	info!("Levels stats command processed successfully");
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
		_ => 155001, // Cap at level 20
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
		_ => 999999, // No next level after 20
	}
}

async fn create_progress_bar(
	current: i128, max: i128, user_color: Option<Colour>,
) -> Result<(CommandFiles, i32)> {
	// Calculate percentage
	let percent = if max > 0 {
		((current as f64 / max as f64) * 100.0) as i32
	} else {
		100
	};

	// Ensure percentage is between 0 and 100
	let percent = percent.max(0).min(100);

	let color = user_color.unwrap_or(COLOR);
	let rgb_color = [color.r(), color.b(), color.g(), 255];

	// Generate the progress bar image with the user's color or default color
	let image_data = generate_progress_bar_image_in_memory(percent as u32, rgb_color)
		.map_err(|e| anyhow!("Failed to generate progress bar image: {}", e))?;

	// Generate a unique filename for the attachment
	let uuid = Uuid::new_v4();
	let filename = format!("progress_{}.png", uuid);

	// Create the CommandFiles object
	let file = CommandFiles::new(filename.clone(), image_data);

	Ok((file, percent))
}
