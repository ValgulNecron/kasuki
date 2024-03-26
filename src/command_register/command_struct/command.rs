use crate::command_register::command_struct::common::{Arg, DefaultPermission, Localised};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Command {
    pub name: String,
    pub desc: String,
    pub dm_command: bool,
    pub nsfw: bool,
    pub localised: Option<Vec<Localised>>,
    pub args: Option<Vec<Arg>>,
    pub permissions: Option<Vec<DefaultPermission>>,
}
