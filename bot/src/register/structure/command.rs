use serde::{Deserialize, Serialize};

use crate::register::structure::common::{
	Arg, CommandInstallationContext, CommandIntegrationContext, DefaultPermission, Localised,
};

#[derive(Debug, Serialize, Deserialize, Clone)]

pub struct Command {
	pub name: String,
	pub desc: String,
	pub integration_context: CommandIntegrationContext,
	pub installation_context: CommandInstallationContext,
	pub nsfw: bool,
	pub permissions: Option<Vec<DefaultPermission>>,
	pub args: Option<Vec<Arg>>,
	pub localised: Option<Vec<Localised>>,
}
