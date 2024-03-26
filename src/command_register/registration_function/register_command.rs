use crate::command_register::command_struct::command::Command;
use crate::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use serenity::all::{CommandData, CreateCommand, Http};
use std::fs;
use std::io::BufReader;
use std::sync::Arc;
use tracing::error;

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

fn get_commands(path: &str) -> Result<Vec<Command>, AppError> {
    let mut commands = Vec::new();
    let paths = fs::read_dir(path)?;
    for path in paths {
        let path = path?;
        let file = fs::File::open(path.path()).map_err(|e| AppError {
            message: format!("Failed to open file: {:?} with error {}", path.path(), e),
            error_type: ErrorType::File,
            error_response_type: ErrorResponseType::None,
        })?;
        let reader = BufReader::new(file);
        let command: Command = serde_json::from_reader(reader).map_err(|e| AppError {
            message: format!("Failed to parse file: {:?} with error {}", path.path(), e),
            error_type: ErrorType::File,
            error_response_type: ErrorResponseType::None,
        })?;
        commands.push(command);
    }
    Ok(commands)
}

async fn create_command(command: &Command, http: &Arc<Http>) {
    let mut command_build = CreateCommand::new(&command.name)
        .nsfw(command.nsfw)
        .dm_permission(command.dm_command)
        .description(&command.desc);

    command_build = match command.args {
        Some(args) => {}
        None => command_build,
    }
}
