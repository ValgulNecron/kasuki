use std::sync::Arc;

use serenity::all::{Command, Http};
use tracing::{error, info, trace};

use crate::command_register::registration_function::register_command::creates_commands;
use crate::command_register::registration_function::register_message_command::creates_message_command;
use crate::command_register::registration_function::register_subcommand::creates_subcommands;
use crate::command_register::registration_function::register_subcommand_group::creates_subcommands_group;
use crate::command_register::registration_function::register_user_command::creates_user_command;

/// This asynchronous function dispatches the creation and deletion of commands in Discord.
///
/// If the `is_ok` parameter is true, it first calls the `delete_command` function to delete all existing global commands in Discord.
///
/// It then logs a message indicating that it is starting to create commands.
///
/// It calls the `creates_commands`, `creates_subcommands`, `creates_subcommands_group`, and `creates_user_command` functions in order to create global commands, subcommands, subcommand groups, and user commands in Discord, respectively.
///
/// Finally, it logs a message indicating that it has finished creating commands.
///
/// # Arguments
///
/// * `http` - An `Arc<Http>` instance used to send the commands to the Discord API.
/// * `is_ok` - A boolean indicating whether to delete all existing global commands in Discord before creating new ones.
pub async fn command_dispatcher(http: &Arc<Http>, is_ok: bool) {
    if is_ok {
        delete_command(http).await;
    }
    info!("Starting to create commands...");

    let start = std::time::Instant::now();
    creates_commands(http).await;
    creates_subcommands(http).await;
    creates_subcommands_group(http).await;
    creates_user_command(http).await;
    creates_message_command(http).await;
    let duration = start.elapsed();
    info!("Time taken to create commands: {:?}", duration);

    info!("Done creating commands")
}

/// This asynchronous function deletes all existing global commands in Discord.
///
/// It first logs a message indicating that it is starting to delete commands.
///
/// It then retrieves all existing global commands in Discord and iterates over each one.
/// For each command, it logs a trace message indicating that it is removing the command and then calls the `delete_global_command` function to delete the command.
/// If an error occurs during this process, it logs the error and returns early.
///
/// Finally, it logs a message indicating that it has finished deleting commands.
///
/// # Arguments
///
/// * `http` - An `Arc<Http>` instance used to send the delete command requests to the Discord API.
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
