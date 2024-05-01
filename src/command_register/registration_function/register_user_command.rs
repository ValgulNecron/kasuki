use std::fs;
use std::io::BufReader;
use std::sync::Arc;

use serenity::all::{CommandType, CreateCommand, Http, Permissions};
use tracing::{error, trace};

use crate::command_register::command_struct::subcommand_group::SubCommandGroup;
use crate::command_register::command_struct::user_command::UserCommand;
use crate::command_register::registration_function::common::get_permission;
use crate::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};

/// This asynchronous function creates user commands in Discord by reading from a JSON file and sending them to the Discord API.
///
/// It first calls the `get_user_command` function to read the user commands from the JSON file located at "./json/user_command".
/// If an error occurs during this process, it logs the error and returns early.
/// If the user commands are successfully read, it iterates over each user command and calls the `create_command` function to send the user command to the Discord API.
///
/// # Arguments
///
/// * `http` - An `Arc<Http>` instance used to send the user commands to the Discord API.
pub async fn creates_user_command(http: &Arc<Http>) {
    let commands = match get_user_command("./json/user_command") {
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

/// This function reads user commands from a JSON file located at the given path and returns them as a vector of `UserCommand` structs.
///
/// It first reads the directory at the given path and maps any errors to an `AppError`.
/// It then iterates over each entry in the directory.
/// If an entry is a file with a ".json" extension, it opens the file and reads it into a `UserCommand` struct.
/// If an error occurs during this process, it maps the error to an `AppError`.
/// If the user command is successfully read, it is pushed to the `subcommands_group` vector.
/// If no user commands are found in the directory, it logs a trace message.
///
/// # Arguments
///
/// * `path` - A string slice representing the path to the directory containing the JSON files.
///
/// # Returns
///
/// A `Result` containing either a vector of `UserCommand` structs if the user commands are successfully read, or an `AppError` if an error occurs.
fn get_user_command(path: &str) -> Result<Vec<UserCommand>, AppError> {
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
            let command: UserCommand = serde_json::from_reader(reader).map_err(|e| AppError {
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
        trace!("No user command found in the directory: {:?}", path);
    }
    Ok(subcommands_group)
}

/// This asynchronous function creates a global user command in Discord using the provided `UserCommand` struct and `Http` instance.
///
/// It first creates a `CreateCommand` instance using the name from the `UserCommand` struct and sets the command type to `User`.
///
/// It then calls the `get_permission` function to set the default member permissions of the `CreateCommand` based on the permissions in the `UserCommand` struct.
///
/// Finally, it sends the `CreateCommand` to the Discord API to create the global user command. If an error occurs during this process, it logs the error.
///
/// # Arguments
///
/// * `command` - A reference to a `UserCommand` struct containing the details of the user command to be created.
/// * `http` - An `Arc<Http>` instance used to send the user command to the Discord API.
async fn create_command(command: &UserCommand, http: &Arc<Http>) {
    let mut command_build = CreateCommand::new(&command.name)
        .kind(CommandType::User)
        .name(&command.name);

    command_build = get_permission(&command.permissions, command_build);

    let e = http.create_global_command(&command_build).await;
    match e {
        Ok(_) => (),
        Err(e) => {
            error!("Failed to create command: {:?}", e);
        }
    }
}
