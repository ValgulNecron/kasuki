use crate::command_register::command_struct::common::DefaultPermission;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MessageCommand {
    pub name: String,
    pub localised: Option<Vec<crate::command_register::command_struct::user_command::Localised>>,
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
