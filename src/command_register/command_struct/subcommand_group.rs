use serde::{Deserialize, Serialize};

use crate::command_register::command_struct::common::{DefaultPermission, Localised};
use crate::command_register::command_struct::subcommand::Command;

/// The `SubCommandGroup` struct represents a group of subcommands that can be executed by the bot.
/// It is derived from `Debug`, `Serialize`, `Deserialize`, and `Clone` traits.
///
/// # Fields
///
/// * `name` - The name of the subcommand group as a `String`.
/// * `desc` - The description of the subcommand group as a `String`.
/// * `dm_command` - A `bool` indicating whether the subcommand group can be executed in direct messages.
/// * `nsfw` - A `bool` indicating whether the subcommand group is not safe for work.
/// * `subcommands` - An `Option` containing a `Vec` of `SubCommand` which represents the subcommands that the subcommand group can execute.
/// * `command` - An `Option` containing a `Vec` of `Command` which represents the commands that the subcommand group can execute.
/// * `permissions` - An `Option` containing a `Vec` of `DefaultPermission` which represents the permissions required to execute the subcommand group.
/// * `localised` - An `Option` containing a `Vec` of `Localised` which represents the localised versions of the subcommand group.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SubCommandGroup {
    pub name: String,
    pub desc: String,
    pub dm_command: bool,
    pub nsfw: bool,
    pub subcommands: Option<Vec<SubCommand>>,
    pub command: Option<Vec<Command>>,
    pub permissions: Option<Vec<DefaultPermission>>,
    pub localised: Option<Vec<Localised>>,
}

/// The `SubCommand` struct represents a subcommand that can be executed by the bot.
/// It is derived from `Debug`, `Serialize`, `Deserialize`, and `Clone` traits.
///
/// # Fields
///
/// * `name` - The name of the subcommand as a `String`.
/// * `desc` - The description of the subcommand as a `String`.
/// * `localised` - An `Option` containing a `Vec` of `Localised` which represents the localised versions of the subcommand.
/// * `command` - An `Option` containing a `Vec` of `Command` which represents the commands that the subcommand can execute.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SubCommand {
    pub name: String,
    pub desc: String,
    pub localised: Option<Vec<Localised>>,
    pub command: Option<Vec<Command>>,
}
