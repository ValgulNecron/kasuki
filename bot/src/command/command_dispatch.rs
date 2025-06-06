use crate::command::admin::anilist::add_activity::AddActivityCommand;
use crate::command::admin::anilist::delete_activity::DeleteActivityCommand;
use crate::command::admin::server::lang::LangCommand;
use crate::command::admin::server::module::{ModuleCommand, check_activation_status};
use crate::command::ai::image::ImageCommand;
use crate::command::ai::question::QuestionCommand;
use crate::command::ai::transcript::TranscriptCommand;
use crate::command::ai::translation::TranslationCommand;
use crate::command::anilist_server::list_all_activity::ListAllActivity;
use crate::command::anilist_server::list_register_user::ListRegisterUser;
use crate::command::anilist_user::anime::AnimeCommand;
use crate::command::anilist_user::character::CharacterCommand;
use crate::command::anilist_user::compare::CompareCommand;
use crate::command::anilist_user::level::LevelCommand;
use crate::command::anilist_user::ln::LnCommand;
use crate::command::anilist_user::manga::MangaCommand;
use crate::command::anilist_user::random::RandomCommand;
use crate::command::anilist_user::register::RegisterCommand;
use crate::command::anilist_user::search::SearchCommand;
use crate::command::anilist_user::seiyuu::SeiyuuCommand;
use crate::command::anilist_user::staff::StaffCommand;
use crate::command::anilist_user::studio::StudioCommand;
use crate::command::anilist_user::user::UserCommand;
use crate::command::anilist_user::waifu::WaifuCommand;
use crate::command::anime::random_image::AnimeRandomImageCommand;
use crate::command::anime_nsfw::random_nsfw_image::AnimeRandomNsfwImageCommand;
use crate::command::bot::credit::CreditCommand;
use crate::command::bot::info::InfoCommand;
use crate::command::bot::ping::PingCommand;
use crate::command::command::CommandRun;
use crate::command::guess_kind::guess_command_kind;
use crate::command::management::give_premium_sub::GivePremiumSubCommand;
use crate::command::management::kill_switch::KillSwitchCommand;
use crate::command::management::remove_test_sub::RemoveTestSubCommand;
use crate::command::music::clear::ClearCommand;
use crate::command::music::join::JoinCommand;
use crate::command::music::leave::LeaveCommand;
use crate::command::music::pause::PauseCommand;
use crate::command::music::play::PlayCommand;
use crate::command::music::queue::QueueCommand;
use crate::command::music::remove::RemoveCommand;
use crate::command::music::resume::ResumeCommand;
use crate::command::music::seek::SeekCommand;
use crate::command::music::skip::SkipCommand;
use crate::command::music::stop::StopCommand;
use crate::command::music::swap::SwapCommand;
use crate::command::server::generate_image_pfp_server::GenerateImagePfPCommand;
use crate::command::server::generate_image_pfp_server_global::GenerateGlobalImagePfPCommand;
use crate::command::server::guild::GuildCommand;
use crate::command::steam::steam_game_info::SteamGameInfoCommand;
use crate::command::user::avatar::AvatarCommand;
use crate::command::user::banner::BannerCommand;
use crate::command::user::command_usage::CommandUsageCommand;
use crate::command::user::profile::ProfileCommand;
use crate::command::vn::character::VnCharacterCommand;
use crate::command::vn::game::VnGameCommand;
use crate::command::vn::producer::VnProducerCommand;
use crate::command::vn::staff::VnStaffCommand;
use crate::command::vn::stats::VnStatsCommand;
use crate::command::vn::user::VnUserCommand;
use crate::config::DbConfig;
use crate::database;
use crate::database::module_activation::Model;
use crate::database::prelude::ModuleActivation;
use crate::event_handler::BotData;
use crate::get_url;
use anyhow::{Context as AnyhowContext, Result};
use sea_orm::ColumnTrait;
use sea_orm::{EntityTrait, QueryFilter};
use serenity::all::{CommandInteraction, Context as SerenityContext};
use std::error::Error;
use std::time::Instant;
use tracing::{debug, error, info, trace, warn, instrument};

/// Dispatches a command to the appropriate handler based on the command name.
///
/// This function is the central command routing mechanism for the bot. It:
/// 1. Identifies the command type and name from the interaction
/// 2. Creates the appropriate command handler instance
/// 3. Executes the command
/// 4. Records usage statistics
///
/// # Command Routing Architecture
///
/// The function uses a large match statement to route commands to their handlers.
/// Each command follows this pattern:
/// ```
/// CommandHandlerStruct {
///     ctx: ctx.clone(),
///     command_interaction: command_interaction.clone(),
/// }
/// .run_slash()
/// .await?
/// ```
///
/// # Command Naming Convention
///
/// Commands follow a hierarchical naming convention:
/// - First level: Category (e.g., "user", "admin", "bot")
/// - Second level: Subcategory or specific command (e.g., "avatar", "info")
/// 
/// Example: "user_avatar" for the user avatar command
///
/// # Arguments
///
/// * `ctx` - The Serenity context containing bot state and Discord connection
/// * `command_interaction` - The command interaction from Discord containing command data
///
/// # Returns
///
/// * `Ok(())` - If the command was dispatched and executed successfully
/// * `Err(Error)` - If there was an error dispatching or executing the command
#[instrument(name = "dispatch_command", skip(ctx, command_interaction), fields(
	user_id = ?command_interaction.user.id,
	guild_id = ?command_interaction.guild_id,
))]
pub async fn dispatch_command(
	ctx: &SerenityContext, command_interaction: &CommandInteraction,
) -> Result<()> {
	info!("Dispatching command from user: {} (ID: {})", 
		command_interaction.user.name, command_interaction.user.id);

	// Extract bot data from context for command usage tracking
	let bot_data = ctx.data::<BotData>().clone();

	// Determine command type and name from the interaction
	// This uses a helper function to parse the command structure
	let (kind, name) = guess_command_kind(command_interaction);

	// Create a full command name by combining type and name
	// This is used for logging and usage statistics
	let full_command_name = format!("{} {}", kind, name);

	debug!("Command details: type={}, name={}, full_name={}", kind, name, full_command_name);

	// Log whether the command was executed in a guild or DM
	// This is important for commands that behave differently in these contexts
	if let Some(guild_id) = command_interaction.guild_id {
		debug!("Command executed in guild: {}", guild_id);
	} else {
		debug!("Command executed in DM");
	}

	trace!("Command options: {:?}", command_interaction.data.options());

	// Start timing the command execution
	let start_time = Instant::now();
	info!("Executing command: {}", full_command_name);

	// The following match statement routes commands to their appropriate handlers
	// Commands are organized by category (user, admin, ai, etc.)
	// Each case instantiates the appropriate command handler and calls run_slash()
	match name.as_str() {
		// === USER COMMANDS ===
		// Commands related to user profiles and information
		"user_avatar" => {
			trace!("Dispatching user_avatar command");
			let command_start = Instant::now();
			let result = AvatarCommand {
				ctx: ctx.clone(),
				command_interaction: command_interaction.clone(),
			}
			.run_slash()
			.await;
			debug!("user_avatar command execution took {:?}", command_start.elapsed());
			result?
		},
		"user_banner" => {
			trace!("Dispatching user_banner command");
			let command_start = Instant::now();
			let result = BannerCommand {
				ctx: ctx.clone(),
				command_interaction: command_interaction.clone(),
			}
			.run_slash()
			.await;
			debug!("user_banner command execution took {:?}", command_start.elapsed());
			result?
		},
		"user_profile" => {
			trace!("Dispatching user_profile command");
			let command_start = Instant::now();
			let result = ProfileCommand {
				ctx: ctx.clone(),
				command_interaction: command_interaction.clone(),
			}
			.run_slash()
			.await;
			debug!("user_profile command execution took {:?}", command_start.elapsed());
			result?
		},
		"user_command_usage" => {
			trace!("Dispatching user_command_usage command");
			let command_start = Instant::now();
			let result = CommandUsageCommand {
				ctx: ctx.clone(),
				command_interaction: command_interaction.clone(),
			}
			.run_slash()
			.await;
			debug!("user_command_usage command execution took {:?}", command_start.elapsed());
			result?
		},

		// === ADMIN COMMANDS ===
		// Commands for server administration and configuration
		"admin_general_lang" => {
			trace!("Dispatching admin_general_lang command");
			let command_start = Instant::now();
			let result = LangCommand {
				ctx: ctx.clone(),
				command_interaction: command_interaction.clone(),
			}
			.run_slash()
			.await;
			debug!("admin_general_lang command execution took {:?}", command_start.elapsed());
			result?
		},
		"admin_general_module" => {
			trace!("Dispatching admin_general_module command");
			let command_start = Instant::now();
			let result = ModuleCommand {
				ctx: ctx.clone(),
				command_interaction: command_interaction.clone(),
			}
			.run_slash()
			.await;
			debug!("admin_general_module command execution took {:?}", command_start.elapsed());
			result?
		},

		"admin_anilist_add_anime_activity" => {
			trace!("Dispatching admin_anilist_add_anime_activity command");
			let command_start = Instant::now();
			let result = AddActivityCommand {
				ctx: ctx.clone(),
				command_interaction: command_interaction.clone(),
			}
			.run_slash()
			.await;
			debug!("admin_anilist_add_anime_activity command execution took {:?}", command_start.elapsed());
			result?
		},
		"admin_anilist_delete_anime_activity" => {
			trace!("Dispatching admin_anilist_delete_anime_activity command");
			let command_start = Instant::now();
			let result = DeleteActivityCommand {
				ctx: ctx.clone(),
				command_interaction: command_interaction.clone(),
			}
			.run_slash()
			.await;
			debug!("admin_anilist_delete_anime_activity command execution took {:?}", command_start.elapsed());
			result?
		},

		// === STEAM COMMANDS ===
		// Commands for retrieving Steam game information
		"steam_game" => {
			trace!("Dispatching steam_game command");
			let command_start = Instant::now();
			let result = SteamGameInfoCommand {
				ctx: ctx.clone(),
				command_interaction: command_interaction.clone(),
			}
			.run_slash()
			.await;
			debug!("steam_game command execution took {:?}", command_start.elapsed());
			result?
		},

		// === AI COMMANDS ===
		// Commands for AI-powered features like image generation and text processing
		"ai_image" => {
			trace!("Dispatching ai_image command");
			let command_start = Instant::now();
			let result = ImageCommand {
				ctx: ctx.clone(),
				command_interaction: command_interaction.clone(),
				command_name: full_command_name.clone(),
			}
			.run_slash()
			.await;
			debug!("ai_image command execution took {:?}", command_start.elapsed());
			result?
		},
		"ai_question" => {
			QuestionCommand {
				ctx: ctx.clone(),
				command_interaction: command_interaction.clone(),
				command_name: full_command_name.clone(),
			}
			.run_slash()
			.await?
		},
		"ai_transcript" => {
			TranscriptCommand {
				ctx: ctx.clone(),
				command_interaction: command_interaction.clone(),
				command_name: full_command_name.clone(),
			}
			.run_slash()
			.await?
		},
		"ai_translation" => {
			TranslationCommand {
				ctx: ctx.clone(),
				command_interaction: command_interaction.clone(),
				command_name: full_command_name.clone(),
			}
			.run_slash()
			.await?
		},

		// === ANILIST SERVER COMMANDS ===
		// Commands for managing Anilist integrations at the server level
		"list_user" => {
			ListRegisterUser {
				ctx: ctx.clone(),
				command_interaction: command_interaction.clone(),
			}
			.run_slash()
			.await?
		},
		"list_activity" => {
			ListAllActivity {
				ctx: ctx.clone(),
				command_interaction: command_interaction.clone(),
			}
			.run_slash()
			.await?
		},

		// === ANILIST USER COMMANDS ===
		// Commands for interacting with Anilist data at the user level
		"anime" => {
			trace!("Dispatching anime command");
			let command_start = Instant::now();
			let result = AnimeCommand {
				ctx: ctx.clone(),
				command_interaction: command_interaction.clone(),
			}
			.run_slash()
			.await;
			debug!("anime command execution took {:?}", command_start.elapsed());
			result?
		},
		"character" => {
			CharacterCommand {
				ctx: ctx.clone(),
				command_interaction: command_interaction.clone(),
			}
			.run_slash()
			.await?
		},
		"compare" => {
			CompareCommand {
				ctx: ctx.clone(),
				command_interaction: command_interaction.clone(),
			}
			.run_slash()
			.await?
		},
		"level" => {
			LevelCommand {
				ctx: ctx.clone(),
				command_interaction: command_interaction.clone(),
			}
			.run_slash()
			.await?
		},
		"ln" => {
			LnCommand {
				ctx: ctx.clone(),
				command_interaction: command_interaction.clone(),
			}
			.run_slash()
			.await?
		},
		"manga" => {
			MangaCommand {
				ctx: ctx.clone(),
				command_interaction: command_interaction.clone(),
			}
			.run_slash()
			.await?
		},
		"anilist_user" => {
			UserCommand {
				ctx: ctx.clone(),
				command_interaction: command_interaction.clone(),
			}
			.run_slash()
			.await?
		},
		"waifu" => {
			WaifuCommand {
				ctx: ctx.clone(),
				command_interaction: command_interaction.clone(),
			}
			.run_slash()
			.await?
		},
		"random" => {
			RandomCommand {
				ctx: ctx.clone(),
				command_interaction: command_interaction.clone(),
			}
			.run_slash()
			.await?
		},
		"register" => {
			RegisterCommand {
				ctx: ctx.clone(),
				command_interaction: command_interaction.clone(),
			}
			.run_slash()
			.await?
		},
		"staff" => {
			StaffCommand {
				ctx: ctx.clone(),
				command_interaction: command_interaction.clone(),
			}
			.run_slash()
			.await?
		},
		"studio" => {
			StudioCommand {
				ctx: ctx.clone(),
				command_interaction: command_interaction.clone(),
			}
			.run_slash()
			.await?
		},
		"search" => {
			SearchCommand {
				ctx: ctx.clone(),
				command_interaction: command_interaction.clone(),
			}
			.run_slash()
			.await?
		},
		"seiyuu" => {
			SeiyuuCommand {
				ctx: ctx.clone(),
				command_interaction: command_interaction.clone(),
			}
			.run_slash()
			.await?
		},

		// === ANIME COMMANDS ===
		// Commands for retrieving anime images and related content
		"random_anime_random_image" => {
			AnimeRandomImageCommand {
				ctx: ctx.clone(),
				command_interaction: command_interaction.clone(),
			}
			.run_slash()
			.await?
		},

		// === ANIME NSFW COMMANDS ===
		// Age-restricted commands for anime content
		"random_hanime_random_himage" => {
			AnimeRandomNsfwImageCommand {
				ctx: ctx.clone(),
				command_interaction: command_interaction.clone(),
			}
			.run_slash()
			.await?
		},

		// === BOT COMMANDS ===
		// Commands for bot information, status, and credits
		"bot_credit" => {
			trace!("Dispatching bot_credit command");
			let command_start = Instant::now();
			let result = CreditCommand {
				ctx: ctx.clone(),
				command_interaction: command_interaction.clone(),
			}
			.run_slash()
			.await;
			debug!("bot_credit command execution took {:?}", command_start.elapsed());
			result?
		},
		"bot_info" => {
			trace!("Dispatching bot_info command");
			let command_start = Instant::now();
			let result = InfoCommand {
				ctx: ctx.clone(),
				command_interaction: command_interaction.clone(),
			}
			.run_slash()
			.await;
			debug!("bot_info command execution took {:?}", command_start.elapsed());
			result?
		},
		"bot_ping" => {
			trace!("Dispatching bot_ping command");
			let command_start = Instant::now();
			let result = PingCommand {
				ctx: ctx.clone(),
				command_interaction: command_interaction.clone(),
			}
			.run_slash()
			.await;
			debug!("bot_ping command execution took {:?}", command_start.elapsed());
			result?
		},

		// === MANAGEMENT COMMANDS ===
		// Administrative commands for bot owners and managers
		"kill_switch" => {
			KillSwitchCommand {
				ctx: ctx.clone(),
				command_interaction: command_interaction.clone(),
			}
			.run_slash()
			.await?
		},
		"give_premium_sub" => {
			GivePremiumSubCommand {
				ctx: ctx.clone(),
				command_interaction: command_interaction.clone(),
			}
			.run_slash()
			.await?
		},
		"remove_test_sub" => {
			RemoveTestSubCommand {
				ctx: ctx.clone(),
				command_interaction: command_interaction.clone(),
			}
			.run_slash()
			.await?
		},

		// === SERVER COMMANDS ===
		// Commands for server management and customization
		"server_guild" => {
			GuildCommand {
				ctx: ctx.clone(),
				command_interaction: command_interaction.clone(),
			}
			.run_slash()
			.await?
		},
		"server_guild_image" => {
			GenerateImagePfPCommand {
				ctx: ctx.clone(),
				command_interaction: command_interaction.clone(),
			}
			.run_slash()
			.await?
		},
		"server_guild_image_g" => {
			GenerateGlobalImagePfPCommand {
				ctx: ctx.clone(),
				command_interaction: command_interaction.clone(),
			}
			.run_slash()
			.await?
		},

		"vn_game" => {
			VnGameCommand {
				ctx: ctx.clone(),
				command_interaction: command_interaction.clone(),
			}
			.run_slash()
			.await?
		},
		"vn_character" => {
			VnCharacterCommand {
				ctx: ctx.clone(),
				command_interaction: command_interaction.clone(),
			}
			.run_slash()
			.await?
		},
		"vn_staff" => {
			VnStaffCommand {
				ctx: ctx.clone(),
				command_interaction: command_interaction.clone(),
			}
			.run_slash()
			.await?
		},
		"vn_user" => {
			VnUserCommand {
				ctx: ctx.clone(),
				command_interaction: command_interaction.clone(),
			}
			.run_slash()
			.await?
		},
		"vn_producer" => {
			VnProducerCommand {
				ctx: ctx.clone(),
				command_interaction: command_interaction.clone(),
			}
			.run_slash()
			.await?
		},
		"vn_stats" => {
			VnStatsCommand {
				ctx: ctx.clone(),
				command_interaction: command_interaction.clone(),
			}
			.run_slash()
			.await?
		},

		"music_play" => {
			trace!("Dispatching music_play command");
			let command_start = Instant::now();
			let result = PlayCommand {
				ctx: ctx.clone(),
				command_interaction: command_interaction.clone(),
			}
			.run_slash()
			.await;
			debug!("music_play command execution took {:?}", command_start.elapsed());
			result?
		},
		"music_pause" => {
			PauseCommand {
				ctx: ctx.clone(),
				command_interaction: command_interaction.clone(),
			}
			.run_slash()
			.await?
		},
		"music_resume" => {
			ResumeCommand {
				ctx: ctx.clone(),
				command_interaction: command_interaction.clone(),
			}
			.run_slash()
			.await?
		},
		"music_stop" => {
			StopCommand {
				ctx: ctx.clone(),
				command_interaction: command_interaction.clone(),
			}
			.run_slash()
			.await?
		},
		"music_skip" => {
			SkipCommand {
				ctx: ctx.clone(),
				command_interaction: command_interaction.clone(),
			}
			.run_slash()
			.await?
		},
		"music_queue" => {
			trace!("Dispatching music_queue command");
			let command_start = Instant::now();
			let result = QueueCommand {
				ctx: ctx.clone(),
				command_interaction: command_interaction.clone(),
			}
			.run_slash()
			.await;
			debug!("music_queue command execution took {:?}", command_start.elapsed());
			result?
		},
		"music_clear" => {
			ClearCommand {
				ctx: ctx.clone(),
				command_interaction: command_interaction.clone(),
			}
			.run_slash()
			.await?
		},
		"music_remove" => {
			RemoveCommand {
				ctx: ctx.clone(),
				command_interaction: command_interaction.clone(),
			}
			.run_slash()
			.await?
		},
		"music_seek" => {
			SeekCommand {
				ctx: ctx.clone(),
				command_interaction: command_interaction.clone(),
			}
			.run_slash()
			.await?
		},
		"music_swap" => {
			SwapCommand {
				ctx: ctx.clone(),
				command_interaction: command_interaction.clone(),
			}
			.run_slash()
			.await?
		},
		"music_join" => {
			JoinCommand {
				ctx: ctx.clone(),
				command_interaction: command_interaction.clone(),
			}
			.run_slash()
			.await?
		},
		"music_leave" => {
			LeaveCommand {
				ctx: ctx.clone(),
				command_interaction: command_interaction.clone(),
			}
			.run_slash()
			.await?
		},

		_ => {
			error!("Unknown command requested: {}", full_command_name);
			warn!("This could indicate a mismatch between registered commands and implementation");
			debug!("Command options: {:?}", command_interaction.data.options());
			error!("Available commands do not include: {}", name);
			return Err(anyhow::anyhow!("Command not found: {}", full_command_name))
				.context(format!("No handler implemented for command: {}", full_command_name));
		},
	};

	// Calculate and log the total execution time
	let execution_time = start_time.elapsed();
	debug!("Command {} execution took {:?}", full_command_name, execution_time);

	// Log performance warning for slow commands
	if execution_time.as_millis() > 1000 {
		warn!("Command {} took over 1 second to execute: {:?}", full_command_name, execution_time);
	}

	debug!("Incrementing command usage statistics for: {}", full_command_name);
	bot_data
		.increment_command_use_per_command(
			full_command_name.clone(),
			command_interaction.user.id.to_string(),
			command_interaction.user.name.to_string(),
		)
		.await;

	info!("Command {} executed successfully", full_command_name);
	Ok(())
}

pub async fn check_if_module_is_on(
	guild_id: String, module: &str, db_config: DbConfig,
) -> Result<bool> {
	debug!("Checking if module '{}' is enabled for guild {}", module, guild_id);

	debug!("Connecting to database to check module activation");
	let connection = sea_orm::Database::connect(get_url(db_config.clone())).await
		.context(format!("Failed to connect to database to check module activation for guild {}", guild_id))?;
	debug!("Successfully connected to database");

	debug!("Querying module activation status for guild {}", guild_id);
	let row = ModuleActivation::find()
		.filter(database::module_activation::Column::GuildId.eq(guild_id.clone()))
		.one(&connection)
		.await
		.context(format!("Failed to query module activation status for guild {}", guild_id))?;

	let row = match row {
		Some(row) => {
			debug!("Found existing module configuration for guild {}", guild_id);
			row
		},
		None => {
			info!("No module configuration found for guild {}, using defaults", guild_id);
			Model {
				guild_id: guild_id.clone(),
				ai_module: true,
				anilist_module: true,
				game_module: true,
				new_members_module: false,
				anime_module: true,
				vn_module: true,
				updated_at: Default::default(),
			}
		}
	};

	debug!("Checking activation status for module '{}'", module);
	let state = check_activation_status(module, row).await;
	debug!("Module '{}' activation status from database: {}", module, state);

	debug!("Checking kill switch status for module '{}'", module);
	let kill_switch_state = check_kill_switch_status(module, db_config, guild_id.clone()).await
		.context(format!("Failed to check kill switch status for module '{}' in guild {}", module, guild_id))?;

	let final_state = state && kill_switch_state;
	debug!("Kill switch status for module '{}': {}", module, kill_switch_state);
	debug!("Final activation status for module '{}': {}", module, final_state);

	info!("Module '{}' is {} for guild {}", module, if final_state { "enabled" } else { "disabled" }, guild_id);
	Ok(final_state)
}

async fn check_kill_switch_status(
	module: &str, db_config: DbConfig, guild_id: String,
) -> Result<bool> {
	debug!("Checking kill switch status for module '{}' in guild {}", module, guild_id);

	debug!("Connecting to database for kill switch check");
	let connection = sea_orm::Database::connect(get_url(db_config.clone())).await
		.context(format!("Failed to connect to database for kill switch check for module '{}' in guild {}", module, guild_id))?;
	debug!("Successfully connected to database for kill switch check");

	debug!("Querying kill switch status for guild {}", guild_id);
	let row = ModuleActivation::find()
		.filter(database::kill_switch::Column::GuildId.eq(guild_id.clone()))
		.one(&connection)
		.await
		.context(format!("Failed to query kill switch status for module '{}' in guild {}", module, guild_id))?;

	let row = match row {
		Some(row) => {
			debug!("Found existing kill switch configuration for guild {}", guild_id);
			row
		},
		None => {
			info!("No kill switch configuration found for guild {}, using defaults", guild_id);
			Model {
				guild_id,
				ai_module: true,
				anilist_module: true,
				game_module: true,
				new_members_module: false,
				anime_module: true,
				vn_module: true,
				updated_at: Default::default(),
			}
		}
	};

	trace!("Kill switch row data: {:?}", row);

	debug!("Determining final kill switch status for module '{}'", module);
	let status = check_activation_status(module, row).await;
	debug!("Kill switch status for module '{}': {}", module, status);

	Ok(status)
}
