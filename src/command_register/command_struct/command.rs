use serde::{Deserialize, Serialize};

use crate::command_register::command_struct::common::{Arg, DefaultPermission, Localised};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Command {
    pub name: String,
    pub desc: String,
    pub dm_command: bool,
    pub nsfw: bool,
    pub permissions: Option<Vec<DefaultPermission>>,
    pub args: Option<Vec<Arg>>,
    pub localised: Option<Vec<Localised>>,
}
