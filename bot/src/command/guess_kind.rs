use serenity::all::{CommandInteraction, ResolvedValue};
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

pub fn guess_command_kind(command_interaction: &CommandInteraction) -> (CommandKind, String) {
    // get the option list
    let options = &command_interaction.data.options();
    if options.is_empty() {
        return (CommandKind::Command, command_interaction.data.name.clone());
    }

    let option = &options[0];
    let value = &option.value;
    if let ResolvedValue::SubCommand(op) = value {
        let command_name = format!("{}_{}", command_interaction.data.name.clone(), option.name);
        return (CommandKind::Subcommand, command_name);
    }
    if let ResolvedValue::SubCommandGroup(op) = value {
        if let ResolvedValue::SubCommand(_) = &op[0].value {
            let command_name = format!(
                "{}_{}_{}",
                command_interaction.data.name.clone(),
                option.name,
                op[0].name
            );
            return (CommandKind::SubcommandGroup, command_name);
        }
    }
    (CommandKind::Command, command_interaction.data.name.clone())
}
