use std::fs;
use std::io::BufReader;
use std::sync::Arc;
use serenity::all::Http;
use tracing::{error, trace};
use crate::command_register::command_struct::command::Command;
use crate::command_register::command_struct::message_command::MessageCommand;
use crate::command_register::command_struct::user_command::UserCommand;
use crate::command_register::registration_function::common::get_vec;
use crate::helper::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use crate::helper::read_file::read_file_as_string;

pub async fn creates_message_command(http: &Arc<Http>) {
    let commands = match get_message_command("./json/message_command") {
        Err(e) => {
            error!("{:?}", e);
            return;
        }
        Ok(c) => c,
    };

    for command in commands {
        //create_command(&command, http).await;
    }
}

fn get_message_command(path: &str) -> Result<Vec<MessageCommand>, AppError> {
    let commands: Vec<MessageCommand> = get_vec(path)?;
    if commands.is_empty() {
        trace!("No commands found in the directory: {:?}", path);
    }
    Ok(commands)
}