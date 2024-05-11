use std::fs;
use std::io::BufReader;
use std::sync::Arc;

use serenity::all::{CommandType, CreateCommand, Http};
use tracing::{error, trace};

use crate::command_register::command_struct::subcommand::SubCommand;
use crate::command_register::registration_function::common::{
    get_permission, get_subcommand_option,
};
use crate::helper::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};

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
pub(crate) fn get_subcommands(path: &str) -> Result<Vec<SubCommand>, AppError> {
    let mut subcommands = Vec::new();
    let paths = fs::read_dir(path).map_err(|e| AppError {
        message: format!("Failed to read directory: {:?} with error {}", path, e),
        error_type: ErrorType::File,
        error_response_type: ErrorResponseType::None,
    })?;
    for entry in paths {
        let entry = entry.map_err(|e| AppError {
            message: format!("Failed to read path with error {}", e),
            error_type: ErrorType::File,
            error_response_type: ErrorResponseType::None,
        })?;

        let path = entry.path();
        if path.is_file() && path.extension().unwrap_or_default() == "json" {
            let file = fs::File::open(path.as_path()).map_err(|e| AppError {
                message: format!("Failed to open file: {:?} with error {}", path.as_path(), e),
                error_type: ErrorType::File,
                error_response_type: ErrorResponseType::None,
            })?;
            let reader = BufReader::new(file);
            let command: SubCommand = serde_json::from_reader(reader).map_err(|e| AppError {
                message: format!(
                    "Failed to parse file: {:?} with error {}",
                    path.as_path(),
                    e
                ),
                error_type: ErrorType::File,
                error_response_type: ErrorResponseType::None,
            })?;
            subcommands.push(command);
        }
    }
    if subcommands.is_empty() {
        trace!("No subcommands found in the directory: {:?}", path);
    }
    Ok(subcommands)
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
        .dm_permission(command.dm_command)
        .description(&command.desc);

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
