use crate::command_register::command_struct::common::{DefaultPermission, Localised};
use crate::command_register::command_struct::subcommand::Command;
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SubCommand {
    pub name: String,
    pub desc: String,
    pub localised: Option<Vec<Localised>>,
    pub command: Option<Vec<Command>>,
}
