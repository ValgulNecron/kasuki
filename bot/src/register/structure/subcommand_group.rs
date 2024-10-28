use serde::{Deserialize, Serialize};

use crate::register::structure::common::{
	CommandInstallationContext, CommandIntegrationContext, DefaultPermission, Localised,
};
use crate::register::structure::subcommand::Command;

#[derive(Debug, Serialize, Deserialize, Clone)]

pub struct SubCommandGroup {
	pub name: String,
	pub desc: String,
	pub integration_context: CommandIntegrationContext,
	pub installation_context: CommandInstallationContext,

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
