use crate::command_register::registration_function::register_command::creates_commands;
use crate::command_register::registration_function::register_subcommand::creates_subcommands;
use serenity::all::{Command, Http};
use std::sync::Arc;
use tracing::{error, info, trace};
use crate::command_register::registration_function::register_subcommand_group::creates_subcommands_group;

pub async fn command_dispatcher(http: &Arc<Http>, is_ok: bool) {
    info!("Starting to create commands...");
    if is_ok {
        delete_command(http).await;
    }

    creates_commands(http).await;
    creates_subcommands(http).await;
    creates_subcommands_group(http).await;

    info!("Done creating commands")
}

async fn delete_command(http: &Arc<Http>) {
    info!("Started deleting command");
    let cmds = Command::get_global_commands(http).await.unwrap();
    for cmd in cmds {
        trace!("Removing {:?}", cmd.name);
        match Command::delete_global_command(http, cmd.id).await {
            Ok(res) => res,
            Err(e) => {
                error!("{} for command {}", e, cmd.name);
                return;
            }
        };
    }
    info!("Done deleting command")
}
