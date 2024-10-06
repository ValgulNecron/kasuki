use std::error::Error;
use std::sync::Arc;

use serenity::all::{CommandType, CreateCommand, Http};
use tracing::{error, trace};

use crate::register::function::common::{
    get_permission, get_subcommand_option, get_vec, get_vec_installation_context,
    get_vec_integration_context,
};
use crate::register::structure::subcommand::SubCommand;

/// This asynchronous function creates subcommands in Discord by reading from a JSON file and sending them to the Discord API.
///
/// It first calls the `get_subcommands` function to read the subcommands from the JSON file located at "./json/subcommand".
/// If an error occurs during this process, it logs the error and returns early.
/// If the subcommands are successfully read, it iterates over each subcommand and calls the `create_command` function to send the subcommand to the Discord API.
///
/// # Arguments
///
/// * `http` - An `Arc<Http>` instance used to send the subcommands to the Discord API.

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

/// This function reads subcommands from a JSON file located at the given path and returns them as a vector of `SubCommand` structs.
///
/// It first reads the directory at the given path and maps any errors to an `AppError`.
/// It then iterates over each entry in the directory.
/// If an entry is a file with a ".json" extension, it opens the file and reads it into a `SubCommand` struct.
/// If an error occurs during this process, it maps the error to an `AppError`.
/// If the subcommand is successfully read, it is pushed to the `subcommands` vector.
/// If no subcommands are found in the directory, it logs a trace message.
///
/// # Arguments
///
/// * `path` - A string slice representing the path to the directory containing the JSON files.
///
/// # Returns
///
/// A `Result` containing either a vector of `SubCommand` structs if the subcommands are successfully read, or an `AppError` if an error occurs.

pub fn get_subcommands(path: &str) -> Result<Vec<SubCommand>, Box<dyn Error>> {
    let commands: Vec<SubCommand> = get_vec(path)?;

    if commands.is_empty() {
        trace!("No commands found in the directory: {:?}", path);
    }

    Ok(commands)
}

/// This asynchronous function creates a global subcommand in Discord using the provided `SubCommand` struct and `Http` instance.
///
/// It first creates a `CreateCommand` instance using the name, NSFW status, command type, DM permission status, and description from the `SubCommand` struct.
///
/// It then calls the `get_permission` function to set the default member permissions of the `CreateCommand` based on the permissions in the `SubCommand` struct.
///
/// If the `SubCommand` struct contains a command, it calls the `get_subcommand_option` function to convert it into command options and sets them on the `CreateCommand`.
///
/// Finally, it sends the `CreateCommand` to the Discord API to create the global subcommand. If an error occurs during this process, it logs the error.
///
/// # Arguments
///
/// * `command` - A reference to a `SubCommand` struct containing the details of the subcommand to be created.
/// * `http` - An `Arc<Http>` instance used to send the subcommand to the Discord API.

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
