use serenity::all::{CommandInteraction, ResolvedValue};
use small_fixed_array::FixedString;
use std::fmt::Display;

pub enum CommandKind {
	Command,
	Subcommand,
	SubcommandGroup,
}

impl Display for CommandKind {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			CommandKind::Command => write!(f, "Command"),
			CommandKind::Subcommand => write!(f, "Subcommand"),
			CommandKind::SubcommandGroup => write!(f, "SubcommandGroup"),
		}
	}
}

/// Determines the command hierarchy from Discord's resolved interaction data.
/// Discord packs subcommands/groups as the first option — we unpack that to build
/// the flat dispatch key (e.g. "anime_search") used by the registry HashMap.
pub fn guess_command_kind(command_interaction: &CommandInteraction) -> (CommandKind, FixedString) {
	let options = &command_interaction.data.options();

	// No options means a top-level command (no subcommand nesting)
	if options.is_empty() {
		return (CommandKind::Command, command_interaction.data.name.clone());
	}

	// Discord always puts the subcommand/group as options[0] — regular args come after
	let option = &options[0];

	let value = &option.value;

	// Single-level nesting: /parent subcommand → dispatch key "parent_subcommand"
	if let ResolvedValue::SubCommand(_) = value {
		let command_name = format!("{}_{}", command_interaction.data.name.clone(), option.name);

		return (
			CommandKind::Subcommand,
			FixedString::from_string_trunc(command_name),
		);
	}

	// Two-level nesting: /parent group subcommand → dispatch key "parent_group_subcommand"
	// The group's first option is always the actual subcommand
	if let ResolvedValue::SubCommandGroup(op) = value {
		if let ResolvedValue::SubCommand(_) = &op[0].value {
			let command_name = format!(
				"{}_{}_{}",
				command_interaction.data.name.clone(),
				option.name,
				op[0].name
			);

			return (
				CommandKind::SubcommandGroup,
				FixedString::from_string_trunc(command_name),
			);
		}
	}

	// Fallback: treat as plain command if options exist but aren't sub/group (e.g. regular args)
	(CommandKind::Command, command_interaction.data.name.clone())
}
