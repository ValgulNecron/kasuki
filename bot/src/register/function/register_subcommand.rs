use std::error::Error;
use std::sync::Arc;

use serenity::all::{CommandType, CreateCommand, Http};
use tracing::{error, trace};

use crate::register::function::common::{
    get_permission, get_subcommand_option, get_vec, get_vec_installation_context,
    get_vec_integration_context,
};
use crate::register::structure::subcommand::SubCommand;
use anyhow::Result;

pub async fn creates_subcommands(http: &Arc<Http>) {
    let commands = match get_subcommands("./json/subcommand") {
        Err(e) => {
            error!("{:?}", e);

            return;
        }
        Ok(c) => c,
    };

    for command in commands {
        create_command(&command, http).await;
    }
}

pub fn get_subcommands(path: &str) -> Result<Vec<SubCommand>> {
    let commands: Vec<SubCommand> = get_vec(path)?;

    if commands.is_empty() {
        trace!("No commands found in the directory: {:?}", path);
    }

    Ok(commands)
}

async fn create_command(command: &SubCommand, http: &Arc<Http>) {
    let mut command_build = CreateCommand::new(&command.name)
        .nsfw(command.nsfw)
        .kind(CommandType::ChatInput)
        .contexts(get_vec_integration_context(&command.integration_context))
        .description(&command.desc)
        .integration_types(get_vec_installation_context(&command.installation_context));

    command_build = get_permission(&command.permissions, command_build);

    command_build = match &command.command {
        Some(command) => {
            let options = get_subcommand_option(command);

            command_build.set_options(options)
        }
        None => command_build,
    };

    let e = http.create_global_command(&command_build).await;

    match e {
        Ok(_) => (),
        Err(e) => {
            error!("Failed to create command: {:?}", e);
        }
    }
}
