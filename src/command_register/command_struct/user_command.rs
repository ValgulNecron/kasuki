use serde::{Deserialize, Serialize};

use crate::command_register::command_struct::common::{
    CommandInstallationContext, DefaultPermission,
};

/// The `UserCommand` struct represents a user command that can be executed by the bot.
/// It is derived from `Debug`, `Serialize`, `Deserialize`, and `Clone` traits.
///
/// # Fields
///
/// * `name` - The name of the user command as a `String`.
/// * `localised` - An `Option` containing a `Vec` of `Localised` which represents the localised versions of the user command.
/// * `permissions` - An `Option` containing a `Vec` of `DefaultPermission` which represents the permissions required to execute the user command.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserCommand {
    pub name: String,
    pub localised: Option<Vec<Localised>>,
    pub installation_context: CommandInstallationContext,
    pub permissions: Option<Vec<DefaultPermission>>,
}

/// The `Localised` struct represents a localised version of a user command.
/// It is derived from `Debug`, `Serialize`, `Deserialize`, and `Clone` traits.
///
/// # Fields
///
/// * `code` - The language code as a `String`.
/// * `name` - The name in the localised language as a `String`.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Localised {
    pub code: String,
    pub name: String,
}
