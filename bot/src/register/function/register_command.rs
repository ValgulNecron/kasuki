use std::sync::Arc;

use serenity::all::{CommandType, CreateCommand, Http};
use tracing::{error, trace};

use crate::register::function::common::{
	get_option, get_permission, get_vec, get_vec_installation_context, get_vec_integration_context,
};
use crate::register::structure::command::Command;
use anyhow::Result;

pub async fn creates_commands(http: &Arc<Http>) {
	let commands = match get_commands("./json/command") {
		Err(e) => {
			error!("{:?}", e);

			return;
		},
		Ok(c) => c,
	};

	for command in commands {
		create_command(&command, http).await;
	}
}

pub fn get_commands(path: &str) -> Result<Vec<Command>> {
	let commands: Vec<Command> = get_vec(path)?;

	if commands.is_empty() {
		trace!("No commands found in the directory: {:?}", path);
	}

	Ok(commands)
}

async fn create_command(command: &Command, http: &Arc<Http>) {
	let mut command_build = CreateCommand::new(&command.name)
		.nsfw(command.nsfw)
		.kind(CommandType::ChatInput)
		.contexts(get_vec_integration_context(&command.integration_context))
		.description(&command.desc)
		.integration_types(get_vec_installation_context(&command.installation_context));

	command_build = get_permission(&command.permissions, command_build);

	command_build = match &command.args {
		Some(args) => {
			let options = get_option(args);

			command_build.set_options(options)
		},
		None => command_build,
	};

	if let Some(locale) = &command.localised {
		for locale in locale {
			command_build = command_build
				.name_localized(&locale.code, &locale.name)
				.description_localized(&locale.code, &locale.desc);
		}
	}

	let e = http.create_global_command(&command_build).await;

	match e {
		Ok(_) => (),
		Err(e) => {
			error!("Failed to create command: {:?}", e);
		},
	}
}
