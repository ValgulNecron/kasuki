use std::sync::Arc;

use serenity::all::{CommandType, CreateCommand, Http};
use tracing::{error, trace};

use crate::command_register::command_struct::command::Command;
use crate::command_register::registration_function::common::{
    get_option, get_permission, get_vec, get_vec_installation_context, get_vec_integration_context,
};
use crate::helper::error_management::error_enum::AppError;

/// This asynchronous function creates commands by reading from a JSON file and sending them to the Discord API.
///
/// It first calls the `get_commands` function to read the commands from the JSON file located at "./json/command".
/// If an error occurs during this process, it logs the error and returns early.
/// If the commands are successfully read, it iterates over each command and calls the `create_command` function to send the command to the Discord API.
///
/// # Arguments
///
/// * `http` - An `Arc<Http>` instance used to send the commands to the Discord API.
pub async fn creates_commands(http: &Arc<Http>) {
    let commands = match get_commands("./json/command") {
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

/// This function reads commands from a JSON file located at the given path and returns them as a vector of `Command` structs.
///
/// It first reads the directory at the given path and maps any errors to an `AppError`.
/// It then iterates over each entry in the directory.
/// If an entry is a file with a ".json" extension, it opens the file and reads it into a `Command` struct.
/// If an error occurs during this process, it maps the error to an `AppError`.
/// If the command is successfully read, it is pushed to the `commands` vector.
/// If no commands are found in the directory, it logs a trace message.
///
/// # Arguments
///
/// * `path` - A string slice representing the path to the directory containing the JSON files.
///
/// # Returns
///
/// A `Result` containing either a vector of `Command` structs if the commands are successfully read, or an `AppError` if an error occurs.
pub fn get_commands(path: &str) -> Result<Vec<Command>, AppError> {
    let commands: Vec<Command> = get_vec(path)?;
    if commands.is_empty() {
        trace!("No commands found in the directory: {:?}", path);
    }
    Ok(commands)
}

/// This asynchronous function creates a global command in Discord using the provided `Command` struct and `Http` instance.
///
/// It first creates a `CreateCommand` instance using the name, NSFW status, command type, DM permission status, and description from the `Command` struct.
///
/// It then calls the `get_permission` function to set the default member permissions of the `CreateCommand` based on the permissions in the `Command` struct.
///
/// If the `Command` struct contains arguments, it calls the `get_option` function to convert them into command options and sets them on the `CreateCommand`.
///
/// If the `Command` struct contains localised versions, it sets the localised name and description on the `CreateCommand`.
///
/// Finally, it sends the `CreateCommand` to the Discord API to create the global command. If an error occurs during this process, it logs the error.
///
/// # Arguments
///
/// * `command` - A reference to a `Command` struct containing the details of the command to be created.
/// * `http` - An `Arc<Http>` instance used to send the command to the Discord API.
async fn create_command(command: &Command, http: &Arc<Http>) {
    let mut command_build = CreateCommand::new(&command.name)
        .nsfw(command.nsfw)
        .kind(CommandType::ChatInput)
        .contexts(get_vec_integration_context(&command.integration_context))
        .description(&command.desc)
        .integration_types(get_vec_installation_context(&command.installation_context));

    command_build = get_permission(&command.permissions , command_build);

    command_build = match &command.args {
        Some(args) => {
            let options = get_option(args);
            command_build.set_options(options)
        }
        None => command_build,
    };
    match &command.localised {
        Some(locale) => {
            for locale in locale {
                command_build = command_build
                    .name_localized(&locale.code, &locale.name)
                    .description_localized(&locale.code, &locale.desc);
            }
        }
        None => {}
    }

    let e = http.create_global_command(&command_build).await;
    match e {
        Ok(_) => (),
        Err(e) => {
            error!("Failed to create command: {:?}", e);
        }
    }
}
