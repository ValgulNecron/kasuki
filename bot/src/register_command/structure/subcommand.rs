use serde::{Deserialize, Serialize};

use crate::register::command_struct::common::{
    Arg, CommandInstallationContext, CommandIntegrationContext, DefaultPermission, Localised,
};

/// The `SubCommand` struct represents a subcommand that can be executed by the bot.
/// It is derived from `Debug`, `Serialize`, `Deserialize`, and `Clone` traits.
///
/// # Fields
///
/// * `name` - The name of the subcommand as a `String`.
/// * `desc` - The description of the subcommand as a `String`.
/// * `dm_command` - A `bool` indicating whether the subcommand can be executed in direct messages.
/// * `nsfw` - A `bool` indicating whether the subcommand is not safe for work.
/// * `permissions` - An `Option` containing a `Vec` of `DefaultPermission` which represents the permissions required to execute the subcommand.
/// * `command` - An `Option` containing a `Vec` of `Command` which represents the commands that the subcommand can execute.
/// * `localised` - An `Option` containing a `Vec` of `Localised` which represents the localised versions of the subcommand.
#[derive(Debug, Serialize, Deserialize, Clone)]

pub struct SubCommand {
    pub name: String,
    pub desc: String,
    pub integration_context: CommandIntegrationContext,
    pub installation_context: CommandInstallationContext,

    pub nsfw: bool,
    pub permissions: Option<Vec<DefaultPermission>>,
    pub command: Option<Vec<Command>>,
    pub localised: Option<Vec<Localised>>,
}

/// The `Command` struct represents a command that a subcommand can execute.
/// It is derived from `Debug`, `Serialize`, `Deserialize`, and `Clone` traits.
///
/// # Fields
///
/// * `name` - The name of the command as a `String`.
/// * `desc` - The description of the command as a `String`.
/// * `args` - An `Option` containing a `Vec` of `Arg` which represents the arguments that the command accepts.
/// * `localised` - An `Option` containing a `Vec` of `Localised` which represents the localised versions of the command.
#[derive(Debug, Serialize, Deserialize, Clone)]

pub struct Command {
    pub name: String,
    pub desc: String,
    pub args: Option<Vec<Arg>>,
    pub localised: Option<Vec<Localised>>,
}
