use std::error::Error;
use std::sync::Arc;

use serenity::all::{CommandType, CreateCommand, Http};
use tracing::{error, trace};

use crate::command_register::command_struct::message_command::MessageCommand;
use crate::command_register::registration_function::common::{
    get_permission, get_vec, get_vec_installation_context,
};

pub async fn creates_message_command(http: &Arc<Http>) {
    let commands = match get_message_command("./json/message_command") {
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

fn get_message_command(path: &str) -> Result<Vec<MessageCommand>, Box<dyn Error>> {
    let commands: Vec<MessageCommand> = get_vec(path)?;
    if commands.is_empty() {
        trace!("No commands found in the directory: {:?}", path);
    }
    Ok(commands)
}

async fn create_command(command: &MessageCommand, http: &Arc<Http>) {
    let mut command_build = CreateCommand::new(&command.name)
        .kind(CommandType::Message)
        .name(&command.name)
        .integration_types(get_vec_installation_context(&command.installation_context));

    if let Some(localised) = &command.localised {
        for local in localised {
            command_build = command_build.name_localized(&local.code, &local.name)
        }
    }

    command_build = get_permission(&command.permissions, command_build);

    let e = http.create_global_command(&command_build).await;
    match e {
        Ok(_) => (),
        Err(e) => {
            error!("Failed to create command: {:?}", e);
        }
    }
}
