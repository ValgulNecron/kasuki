use serde::{Deserialize, Serialize};

use crate::command_register::command_struct::common::DefaultPermission;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserCommand {
    pub name: String,
    pub localised: Option<Vec<Localised>>,
    pub permissions: Option<Vec<DefaultPermission>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Localised {
    pub code: String,
    pub name: String,
}
