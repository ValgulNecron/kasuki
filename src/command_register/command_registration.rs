use crate::structure::command_structure::{get_commands, CommandData};
use log::{error, trace};
use serenity::all::{Command, CreateCommand, CreateCommandOption, Http};
use std::sync::Arc;

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

async fn create_command(command: &CommandData, http: &Arc<Http>) {
    let mut build = CreateCommand::new(&command.name).description(&command.desc);
    match &command.localised {
        Some(localiseds) => {
            for localised in localiseds {
                build = build
                    .name_localized(&localised.code, &localised.name)
                    .description_localized(&localised.code, &localised.desc)
            }
        }
        None => {}
    }

    if &command.arg_num > &(0u32) {
        let options = create_option(command).await;
        for option in options {
            build = build.add_option(option);
        }
    }
    trace!("{:?}", build);
    match Command::create_global_command(http, build).await {
        Ok(res) => res,
        Err(e) => {
            error!("{} for command {}", e, command.name);
            return;
        }
    };
}

async fn create_option(command: &CommandData) -> Vec<CreateCommandOption> {
    let mut options_builds = Vec::new();
    for option in command.args.as_ref().unwrap() {
        let command_type = option.command_type.clone().into();
        let mut options_build = CreateCommandOption::new(command_type, &option.name, &option.desc)
            .required(option.required);
        match &option.choices {
            Some(choices) => {
                for choice in choices {
                    options_build = options_build
                        .add_string_choice(&choice.option_choice, &choice.option_choice);
                }
            }
            None => {}
        }
        match &option.localised_args {
            Some(localiseds) => {
                for localised in localiseds {
                    options_build = options_build
                        .name_localized(&localised.code, &localised.name)
                        .description_localized(&localised.code, &localised.desc)
                }
            }
            None => {}
        }
        options_builds.push(options_build)
    }

    return options_builds;
}
