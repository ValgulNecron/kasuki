use serde::{Deserialize, Serialize};

use crate::register::structure::common::{Arg, DefaultPermission, Localised};

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
