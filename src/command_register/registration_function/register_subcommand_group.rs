use std::fs;
use std::io::BufReader;
use std::sync::Arc;

use serenity::all::{CommandType, CreateCommand, Http};
use tracing::{error, trace};

use crate::command_register::command_struct::subcommand_group::SubCommandGroup;
use crate::command_register::registration_function::common::{
    get_permission, get_subcommand_group_option, get_subcommand_option,
};
use crate::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};

/// This asynchronous function creates subcommand groups in Discord by reading from a JSON file and sending them to the Discord API.
///
/// It first calls the `get_subcommands_group` function to read the subcommand groups from the JSON file located at "./json/subcommand_group".
/// If an error occurs during this process, it logs the error and returns early.
/// If the subcommand groups are successfully read, it iterates over each subcommand group and calls the `create_command` function to send the subcommand group to the Discord API.
///
/// # Arguments
///
/// * `http` - An `Arc<Http>` instance used to send the subcommand groups to the Discord API.
pub async fn creates_subcommands_group(http: &Arc<Http>) {
    let commands = match get_subcommands_group("./json/subcommand_group") {
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

/// This function reads subcommand groups from a JSON file located at the given path and returns them as a vector of `SubCommandGroup` structs.
///
/// It first reads the directory at the given path and maps any errors to an `AppError`.
/// It then iterates over each entry in the directory.
/// If an entry is a file with a ".json" extension, it opens the file and reads it into a `SubCommandGroup` struct.
/// If an error occurs during this process, it maps the error to an `AppError`.
/// If the subcommand group is successfully read, it is pushed to the `subcommands_group` vector.
/// If no subcommand groups are found in the directory, it logs a trace message.
///
/// # Arguments
///
/// * `path` - A string slice representing the path to the directory containing the JSON files.
///
/// # Returns
///
/// A `Result` containing either a vector of `SubCommandGroup` structs if the subcommand groups are successfully read, or an `AppError` if an error occurs.
fn get_subcommands_group(path: &str) -> Result<Vec<SubCommandGroup>, AppError> {
    let mut subcommands_group = Vec::new();
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
            let command: SubCommandGroup =
                serde_json::from_reader(reader).map_err(|e| AppError {
                    message: format!(
                        "Failed to parse file: {:?} with error {}",
                        path.as_path(),
                        e
                    ),
                    error_type: ErrorType::File,
                    error_response_type: ErrorResponseType::None,
                })?;
            subcommands_group.push(command);
        }
    }
    if subcommands_group.is_empty() {
        trace!("No subcommands group found in the directory: {:?}", path);
    }
    Ok(subcommands_group)
}

/// This asynchronous function creates a global subcommand group in Discord using the provided `SubCommandGroup` struct and `Http` instance.
///
/// It first creates a `CreateCommand` instance using the name, NSFW status, command type, DM permission status, and description from the `SubCommandGroup` struct.
///
/// It then calls the `get_permission` function to set the default member permissions of the `CreateCommand` based on the permissions in the `SubCommandGroup` struct.
///
/// If the `SubCommandGroup` struct contains a command, it calls the `get_subcommand_option` function to convert it into command options and sets them on the `CreateCommand`.
///
/// If the `SubCommandGroup` struct contains subcommands, it calls the `get_subcommand_group_option` function to convert them into command options and sets them on the `CreateCommand`.
///
/// Finally, it sends the `CreateCommand` to the Discord API to create the global subcommand group. If an error occurs during this process, it logs the error.
///
/// # Arguments
///
/// * `command` - A reference to a `SubCommandGroup` struct containing the details of the subcommand group to be created.
/// * `http` - An `Arc<Http>` instance used to send the subcommand group to the Discord API.
async fn create_command(command: &SubCommandGroup, http: &Arc<Http>) {
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

    command_build = match &command.subcommands {
        Some(subcommand) => {
            let options = get_subcommand_group_option(subcommand);
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
