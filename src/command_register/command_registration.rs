use crate::structure::command_structure::{get_commands, CommandData};
use log::error;
use serenity::all::{Command, CreateCommand, CreateCommandOption, Http};
use std::sync::Arc;

pub async fn creates_commands(http: &Arc<Http>) {
    let commands = get_commands("./json/command").unwrap();
    for command in commands {
        create_command(&command, http).await;
    }
}

async fn create_command(command: &CommandData, http: &Arc<Http>) {
    let mut build = CreateCommand::new(&command.name).description(&command.desc);
    match &command.localised {
        Some(localiseds) => {
            for localised in localiseds {
                build = build
                    .name_localized(&localised.code, &localised.name)
                    .description(&localised.desc)
            }
        }
        None => {}
    }

    match Command::create_global_command(http, build).await {
        Ok(res) => res,
        Err(e) => {
            error!("{}", e);
            return;
        }
    };
}

async fn create_option(command: &CommandData) -> Vec<CreateCommandOption> {
    let mut options_build = Vec::new();
    for option in command.args.as_ref().unwrap() {
        let command_type = option.command_type.clone().into();
        let mut options_build = CreateCommandOption::new(command_type, &option.name, &option.desc)
            .required(option.required);
        match &command.localised {
            Some(localiseds) => {
                for localised in localiseds {
                    for arg in localised.args.as_ref().unwrap() {
                        options_build = options_build
                            .name_localized(&localised.code, &arg.name)
                            .description_localized(&localised.code, &arg.desc)
                    }
                }
            }
            None => {}
        }
    }
    return options_build;
}
