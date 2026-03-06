use std::borrow::Cow;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use crate::command::registry::{
	all_parent_commands, all_slash_commands, init_registries, CommandMeta, DiscordCommandType,
	GroupDef,
};
use fluent_templates::Loader;
use serenity::all::{
	Command, CommandOptionType, CommandType, CreateCommand, CreateCommandOption, GuildId, Http,
	InstallationContext, InteractionContext, Permissions,
};
use shared::localization::USABLE_LOCALES;
use tracing::{error, info, trace};
use unic_langid::LanguageIdentifier;

/// Discord locale codes matching our FTL directories
const DISCORD_LOCALES: &[&str] = &["en-US", "fr", "de", "ja"];

fn lookup_locale(locale_code: &str, key: &str) -> Option<String> {
	let lang_id = LanguageIdentifier::from_str(locale_code).ok()?;
	USABLE_LOCALES.try_lookup(&lang_id, key)
}

/// Derive the FTL key prefix for a command, handling name collisions
/// by trying `cmd-{name}` first, then `cmd-{parent}_{name}` for subcommands.
fn cmd_ftl_prefix(meta: &CommandMeta) -> String {
	let simple_key = format!("cmd-{}-name", meta.name);
	// Check if the simple key exists in en-US
	if lookup_locale("en-US", &simple_key).is_some() {
		format!("cmd-{}", meta.name)
	} else {
		// Try parent-prefixed version for subcommands
		match meta.command_type {
			DiscordCommandType::SubCommand { parent } => {
				format!("cmd-{}_{}", parent, meta.name)
			},
			DiscordCommandType::SubCommandGroup { parent, .. } => {
				format!("cmd-{}_{}", parent, meta.name)
			},
			_ => format!("cmd-{}", meta.name),
		}
	}
}

/// Derive the FTL key prefix for an arg, handling name collisions.
fn arg_ftl_prefix(cmd_name: &str, arg_name: &str, meta: &CommandMeta) -> String {
	let simple_key = format!("arg-{}-{}-name", cmd_name, arg_name);
	if lookup_locale("en-US", &simple_key).is_some() {
		format!("arg-{}-{}", cmd_name, arg_name)
	} else {
		// Try parent-prefixed version
		match meta.command_type {
			DiscordCommandType::SubCommand { parent } => {
				format!("arg-{}_{}-{}", parent, cmd_name, arg_name)
			},
			DiscordCommandType::SubCommandGroup { parent, .. } => {
				format!("arg-{}_{}-{}", parent, cmd_name, arg_name)
			},
			_ => format!("arg-{}-{}", cmd_name, arg_name),
		}
	}
}

/// Derive the FTL key prefix for a choice.
fn choice_ftl_prefix(
	cmd_name: &str, arg_name: &str, choice_name: &str, meta: &CommandMeta,
) -> String {
	let simple_key = format!("choice-{}-{}-{}-name", cmd_name, arg_name, choice_name);
	if lookup_locale("en-US", &simple_key).is_some() {
		format!("choice-{}-{}-{}", cmd_name, arg_name, choice_name)
	} else {
		match meta.command_type {
			DiscordCommandType::SubCommand { parent } => {
				format!(
					"choice-{}_{}-{}-{}",
					parent, cmd_name, arg_name, choice_name
				)
			},
			DiscordCommandType::SubCommandGroup { parent, .. } => {
				format!(
					"choice-{}_{}-{}-{}",
					parent, cmd_name, arg_name, choice_name
				)
			},
			_ => format!("choice-{}-{}-{}", cmd_name, arg_name, choice_name),
		}
	}
}

pub async fn command_registration(http: &Arc<Http>, is_ok: bool) {
	if is_ok {
		delete_commands(http).await;
	}
	info!("Starting to create commands...");

	let start = std::time::Instant::now();

	// Initialize the registries
	init_registries();

	// Register top-level commands (no parent)
	register_top_level_commands(http).await;

	// Register subcommand parents (which contain their subcommands)
	register_parent_commands(http).await;

	// Register user context menu commands
	register_user_commands(http).await;

	// Register message context menu commands
	register_message_commands(http).await;

	// Register guild-specific commands
	register_guild_commands(http).await;

	let duration = start.elapsed();
	info!("Time taken to create commands: {:?}", duration);
	info!("Done creating commands");
}

async fn delete_commands(http: &Arc<Http>) {
	info!("Started deleting commands");

	let cmds = match Command::get_global_commands(http).await {
		Ok(res) => res,
		Err(e) => {
			error!("could not get the commands: {:#?}", e);
			return;
		},
	};

	for cmd in cmds {
		trace!("Removing {:?}", cmd.name);
		if let Err(e) = Command::delete_global_command(http, cmd.id).await {
			error!("{} for command {}", e, cmd.name);
			return;
		}
	}

	// Collect guild IDs from guild-specific commands
	let mut guild_ids: Vec<u64> = Vec::new();
	for cmd in all_slash_commands() {
		if let DiscordCommandType::GuildChatInput { guild_id } = cmd.meta().command_type {
			if !guild_ids.contains(&guild_id) {
				guild_ids.push(guild_id);
			}
		}
	}

	for guild_id in guild_ids {
		let guild_id = GuildId::from(guild_id);
		let cmds = match http.get_guild_commands(guild_id).await {
			Ok(res) => res,
			Err(e) => {
				error!("could not get the guild commands: {:#?}", e);
				return;
			},
		};

		for cmd in cmds {
			trace!("Removing guild command {:?}", cmd.name);
			if let Err(e) = http.delete_guild_command(guild_id, cmd.id).await {
				error!("{} for command {}", e, cmd.id);
				return;
			}
		}
	}

	info!("Done deleting commands");
}

/// Register top-level ChatInput commands (those with DiscordCommandType::ChatInput)
async fn register_top_level_commands(http: &Arc<Http>) {
	for cmd in all_slash_commands() {
		let meta = cmd.meta();
		if let DiscordCommandType::ChatInput = meta.command_type {
			let command_build = build_chat_input_command(meta);
			if let Err(e) = http.create_global_command(&command_build).await {
				error!("Failed to create command '{}': {:?}", meta.name, e);
			}
		}
	}
}

/// Register parent commands that group subcommands (and subcommand groups)
async fn register_parent_commands(http: &Arc<Http>) {
	// Collect all subcommands grouped by parent name
	let mut subcommands_by_parent: HashMap<&str, Vec<&CommandMeta>> = HashMap::new();
	// Collect subcommand group commands: parent -> group -> commands
	let mut group_commands: HashMap<&str, HashMap<&str, Vec<&CommandMeta>>> = HashMap::new();

	for cmd in all_slash_commands() {
		let meta = cmd.meta();
		match meta.command_type {
			DiscordCommandType::SubCommand { parent } => {
				subcommands_by_parent.entry(parent).or_default().push(meta);
			},
			DiscordCommandType::SubCommandGroup { parent, group } => {
				group_commands
					.entry(parent)
					.or_default()
					.entry(group)
					.or_default()
					.push(meta);
			},
			_ => {},
		}
	}

	for parent in all_parent_commands() {
		let mut command_build = CreateCommand::new(parent.name)
			.kind(CommandType::ChatInput)
			.nsfw(parent.nsfw)
			.description(parent.desc);

		// Set contexts
		let contexts: Vec<InteractionContext> =
			parent.contexts.iter().map(|c| (*c).into()).collect();
		command_build = command_build.contexts(contexts);

		// Set install contexts
		let install_types: Vec<InstallationContext> = parent
			.install_contexts
			.iter()
			.map(|c| (*c).into())
			.collect();
		command_build = command_build.integration_types(install_types);

		// Set permissions
		command_build = apply_permissions(parent.permissions, command_build);

		// Set locales from FTL
		for locale_code in DISCORD_LOCALES {
			if let Some(name) = lookup_locale(locale_code, &format!("parent-{}-name", parent.name))
			{
				command_build = command_build.name_localized(*locale_code, name);
			}
			if let Some(desc) = lookup_locale(locale_code, &format!("parent-{}-desc", parent.name))
			{
				command_build = command_build.description_localized(*locale_code, desc);
			}
		}

		let mut options: Vec<CreateCommandOption<'_>> = Vec::new();

		// Add subcommand group options
		if !parent.groups.is_empty() {
			for group_def in parent.groups {
				let group_option =
					build_subcommand_group_option(group_def, &group_commands, parent.name);
				options.push(group_option);
			}
		}

		// Add direct subcommand options
		if let Some(subs) = subcommands_by_parent.get(parent.name) {
			for meta in subs {
				let sub_option = build_subcommand_option(meta);
				options.push(sub_option);
			}
		}

		command_build = command_build.set_options(options);

		if let Err(e) = http.create_global_command(&command_build).await {
			error!("Failed to create parent command '{}': {:?}", parent.name, e);
		}
	}
}

/// Register user context menu commands
async fn register_user_commands(http: &Arc<Http>) {
	for cmd in all_slash_commands() {
		let meta = cmd.meta();
		if let DiscordCommandType::User = meta.command_type {
			let mut command_build = CreateCommand::new(meta.name).kind(CommandType::User);

			let install_types: Vec<InstallationContext> =
				meta.install_contexts.iter().map(|c| (*c).into()).collect();
			command_build = command_build.integration_types(install_types);

			let prefix = cmd_ftl_prefix(meta);
			for locale_code in DISCORD_LOCALES {
				if let Some(name) = lookup_locale(locale_code, &format!("{}-name", prefix)) {
					command_build = command_build.name_localized(*locale_code, name);
				}
			}

			command_build = apply_permissions(meta.permissions, command_build);

			if let Err(e) = http.create_global_command(&command_build).await {
				error!("Failed to create user command '{}': {:?}", meta.name, e);
			}
		}
	}
}

/// Register message context menu commands
async fn register_message_commands(http: &Arc<Http>) {
	for cmd in all_slash_commands() {
		let meta = cmd.meta();
		if let DiscordCommandType::Message = meta.command_type {
			let mut command_build = CreateCommand::new(meta.name).kind(CommandType::Message);

			let install_types: Vec<InstallationContext> =
				meta.install_contexts.iter().map(|c| (*c).into()).collect();
			command_build = command_build.integration_types(install_types);

			let prefix = cmd_ftl_prefix(meta);
			for locale_code in DISCORD_LOCALES {
				if let Some(name) = lookup_locale(locale_code, &format!("{}-name", prefix)) {
					command_build = command_build.name_localized(*locale_code, name);
				}
			}

			command_build = apply_permissions(meta.permissions, command_build);

			if let Err(e) = http.create_global_command(&command_build).await {
				error!("Failed to create message command '{}': {:?}", meta.name, e);
			}
		}
	}
}

/// Register guild-specific commands
async fn register_guild_commands(http: &Arc<Http>) {
	for cmd in all_slash_commands() {
		let meta = cmd.meta();
		if let DiscordCommandType::GuildChatInput { guild_id } = meta.command_type {
			let command_build = build_guild_command(meta);
			let guild = GuildId::from(guild_id);
			if let Err(e) = http.create_guild_command(guild, &command_build).await {
				error!(
					"Failed to create guild command '{}' for guild {}: {:?}",
					meta.name, guild_id, e
				);
			}
		}
	}
}

// ─── Builder helpers ─────────────────────────────────────────────────────────

fn build_chat_input_command(meta: &CommandMeta) -> CreateCommand<'_> {
	let mut command_build = CreateCommand::new(meta.name)
		.kind(CommandType::ChatInput)
		.nsfw(meta.nsfw)
		.description(meta.desc);

	let contexts: Vec<InteractionContext> = meta.contexts.iter().map(|c| (*c).into()).collect();
	command_build = command_build.contexts(contexts);

	let install_types: Vec<InstallationContext> =
		meta.install_contexts.iter().map(|c| (*c).into()).collect();
	command_build = command_build.integration_types(install_types);

	command_build = apply_permissions(meta.permissions, command_build);

	if !meta.args.is_empty() {
		let options = build_arg_options(meta);
		command_build = command_build.set_options(options);
	}

	let prefix = cmd_ftl_prefix(meta);
	for locale_code in DISCORD_LOCALES {
		if let Some(name) = lookup_locale(locale_code, &format!("{}-name", prefix)) {
			command_build = command_build.name_localized(*locale_code, name);
		}
		if let Some(desc) = lookup_locale(locale_code, &format!("{}-desc", prefix)) {
			command_build = command_build.description_localized(*locale_code, desc);
		}
	}

	command_build
}

fn build_guild_command(meta: &CommandMeta) -> CreateCommand<'_> {
	let mut command_build = CreateCommand::new(meta.name)
		.kind(CommandType::ChatInput)
		.nsfw(meta.nsfw)
		.description(meta.desc);

	command_build = apply_permissions(meta.permissions, command_build);

	if !meta.args.is_empty() {
		let options = build_arg_options(meta);
		command_build = command_build.set_options(options);
	}

	let prefix = cmd_ftl_prefix(meta);
	for locale_code in DISCORD_LOCALES {
		if let Some(name) = lookup_locale(locale_code, &format!("{}-name", prefix)) {
			command_build = command_build.name_localized(*locale_code, name);
		}
		if let Some(desc) = lookup_locale(locale_code, &format!("{}-desc", prefix)) {
			command_build = command_build.description_localized(*locale_code, desc);
		}
	}

	command_build
}

fn build_subcommand_option<'a>(meta: &'a CommandMeta) -> CreateCommandOption<'a> {
	let mut option = CreateCommandOption::new(CommandOptionType::SubCommand, meta.name, meta.desc);

	if !meta.args.is_empty() {
		let arg_options = build_arg_options(meta);
		option = option.set_sub_options(arg_options);
	}

	let prefix = cmd_ftl_prefix(meta);
	for locale_code in DISCORD_LOCALES {
		if let Some(name) = lookup_locale(locale_code, &format!("{}-name", prefix)) {
			option = option.name_localized(*locale_code, name);
		}
		if let Some(desc) = lookup_locale(locale_code, &format!("{}-desc", prefix)) {
			option = option.description_localized(*locale_code, desc);
		}
	}

	option
}

fn build_subcommand_group_option<'a>(
	group_def: &'a GroupDef, group_commands: &'a HashMap<&str, HashMap<&str, Vec<&CommandMeta>>>,
	parent_name: &str,
) -> CreateCommandOption<'a> {
	let mut option = CreateCommandOption::new(
		CommandOptionType::SubCommandGroup,
		group_def.name,
		group_def.desc,
	);

	// Set locales from FTL
	for locale_code in DISCORD_LOCALES {
		if let Some(name) = lookup_locale(
			locale_code,
			&format!("group-{}-{}-name", parent_name, group_def.name),
		) {
			option = option.name_localized(*locale_code, name);
		}
		if let Some(desc) = lookup_locale(
			locale_code,
			&format!("group-{}-{}-desc", parent_name, group_def.name),
		) {
			option = option.description_localized(*locale_code, desc);
		}
	}

	// Add subcommands within this group
	if let Some(parent_groups) = group_commands.get(parent_name) {
		if let Some(cmds) = parent_groups.get(group_def.name) {
			let mut sub_options = Vec::new();
			for meta in cmds {
				sub_options.push(build_subcommand_option(meta));
			}
			option = option.set_sub_options(sub_options);
		}
	}

	option
}

fn build_arg_options<'a>(meta: &'a CommandMeta) -> Vec<CreateCommandOption<'a>> {
	let mut options = Vec::new();

	for arg in meta.args {
		let option_type: CommandOptionType = arg.arg_type.into();
		let mut option = CreateCommandOption::new(option_type, arg.name, arg.desc)
			.required(arg.required)
			.set_autocomplete(arg.autocomplete);

		// Add choices with locale lookups
		for choice in arg.choices {
			let choice_prefix = choice_ftl_prefix(meta.name, arg.name, choice.name, meta);
			let mut choice_locales: HashMap<Cow<'_, str>, Cow<'_, str>> = HashMap::new();
			for locale_code in DISCORD_LOCALES {
				if let Some(name) = lookup_locale(locale_code, &format!("{}-name", choice_prefix)) {
					choice_locales.insert(Cow::Owned(locale_code.to_string()), Cow::Owned(name));
				}
			}
			if choice_locales.is_empty() {
				option = option.add_string_choice(choice.name, choice.name);
			} else {
				option =
					option.add_string_choice_localized(choice.name, choice.name, choice_locales);
			}
		}

		// Add arg locales from FTL
		let arg_prefix = arg_ftl_prefix(meta.name, arg.name, meta);
		for locale_code in DISCORD_LOCALES {
			if let Some(name) = lookup_locale(locale_code, &format!("{}-name", arg_prefix)) {
				option = option.name_localized(*locale_code, name);
			}
			if let Some(desc) = lookup_locale(locale_code, &format!("{}-desc", arg_prefix)) {
				option = option.description_localized(*locale_code, desc);
			}
		}

		options.push(option);
	}

	options
}

fn apply_permissions<'a>(
	permissions: &[crate::command::registry::PermissionType], mut command_build: CreateCommand<'a>,
) -> CreateCommand<'a> {
	if !permissions.is_empty() {
		let mut perm_bit: u64 = 0;
		for perm in permissions {
			let p: Permissions = (*perm).into();
			perm_bit |= p.bits();
		}
		let permission = Permissions::from_bits(perm_bit).unwrap_or(Permissions::empty());
		command_build = command_build.default_member_permissions(permission);
	}
	command_build
}
