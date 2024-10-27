use std::sync::Arc;

use serenity::all::{CommandType, CreateCommand, Http};
use tracing::{error, trace};

use crate::register::function::common::{get_permission, get_vec, get_vec_installation_context};
use crate::register::structure::user_command::UserCommand;
use anyhow::Result;

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

fn get_user_command(path: &str) -> Result<Vec<UserCommand>> {
    let commands: Vec<UserCommand> = get_vec(path)?;

    if commands.is_empty() {
        trace!("No commands found in the directory: {:?}", path);
    }

    Ok(commands)
}

async fn create_command(command: &UserCommand, http: &Arc<Http>) {
    let mut command_build = CreateCommand::new(&command.name)
        .kind(CommandType::User)
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
