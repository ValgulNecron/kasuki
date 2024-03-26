use crate::command_register::command_struct::common::{
    Arg, DefaultPermission, Localised, RemoteCommandOptionType, RemotePermissionType,
};
use crate::command_register::command_struct::subcommand::Command;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SubCommandGroup {
    pub name: String,
    pub desc: String,
    pub localised: Option<Vec<Localised>>,
    pub subcommands: Option<Vec<SubCommand>>,
    pub command: Option<Vec<Command>>,
    #[serde(with = "RemotePermissionType")]
    pub permission: RemotePermissionType,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SubCommand {
    pub name: String,
    pub desc: String,
    pub localised: Option<Vec<Localised>>,
    pub command: Option<Vec<Command>>,
}
