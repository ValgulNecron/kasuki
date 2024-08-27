use serde::{Deserialize, Serialize};

use crate::register::command_struct::common::{CommandInstallationContext, DefaultPermission};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MessageCommand {
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
