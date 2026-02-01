use serde::{Deserialize, Serialize};

use crate::register::structure::common::{CommandInstallationContext, DefaultPermission};

#[derive(Debug, Serialize, Deserialize, Clone)]

pub struct UserCommand {
	pub name: String,
	pub localised: Option<Vec<Localised>>,
	pub installation_context: CommandInstallationContext,
	pub permissions: Option<Vec<DefaultPermission>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]

pub struct Localised {
	pub code: String,
	pub name: String,
}
