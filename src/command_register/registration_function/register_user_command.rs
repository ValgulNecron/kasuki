use crate::command_register::command_struct::subcommand_group::SubCommandGroup;
use crate::command_register::command_struct::user_command::UserCommand;
use crate::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use serenity::all::{CommandType, CreateCommand, Http, Permissions};
use std::fs;
use std::io::BufReader;
use std::sync::Arc;
use tracing::{error, trace};

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

async fn create_command(command: &UserCommand, http: &Arc<Http>) {
    let mut command_build = CreateCommand::new(&command.name)
        .kind(CommandType::User)
        .name(&command.name);

    command_build = match &command.permissions {
        Some(permissions) => {
            let mut perm_bit: u64 = 0;
            for perm in permissions {
                let permission: Permissions = perm.permission.into();
                perm_bit |= permission.bits()
            }
            let permission = Permissions::from_bits(perm_bit).unwrap();
            command_build.default_member_permissions(permission)
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
