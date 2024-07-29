use serde::{Deserialize, Serialize};

use crate::command_register::command_struct::common::{Arg, DefaultPermission, Localised};

/// The `Command` struct represents a command that can be executed by the bot.
/// It is derived from `Debug`, `Serialize`, `Deserialize`, and `Clone` traits.
///
/// # Fields
///
/// * `name` - The name of the command as a `String`.
/// * `desc` - The description of the command as a `String`.
/// * `dm_command` - A `bool` indicating whether the command can be executed in direct messages.
/// * `nsfw` - A `bool` indicating whether the command is not safe for work.
/// * `permissions` - An `Option` containing a `Vec` of `DefaultPermission` which represents the permissions required to execute the command.
/// * `args` - An `Option` containing a `Vec` of `Arg` which represents the arguments that the command accepts.
/// * `localised` - An `Option` containing a `Vec` of `Localised` which represents the localised versions of the command.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GuildCommand {
    pub guild_id: u64,
    pub name: String,
    pub desc: String,
    pub nsfw: bool,
    pub permissions: Option<Vec<DefaultPermission>>,
    pub args: Option<Vec<Arg>>,
    pub localised: Option<Vec<Localised>>,
}
