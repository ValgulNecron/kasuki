use crate::command_register::command_struct::common::{
    Arg, DefaultPermission, Localised, RemoteCommandOptionType, RemotePermissionType,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SubCommand {
    pub name: String,
    pub desc: String,
    pub dm_command: bool,
    pub nsfw: bool,
    pub localised: Option<Vec<Localised>>,
    pub command: Option<Vec<Command>>,
    #[serde(with = "RemotePermissionType")]
    pub permission: RemotePermissionType,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Command {
    pub name: String,
    pub desc: String,
    pub localised: Option<Vec<Localised>>,
    pub args: Option<Vec<Arg>>,
    pub default_permissions: Option<Vec<DefaultPermission>>,
}
