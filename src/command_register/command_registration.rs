use std::env;
use std::sync::Arc;

use serenity::all::{Command, CreateCommand, CreateCommandOption, Http, Permissions};
use tracing::{error, trace};

use crate::command_register::command_structure::{get_commands, CommandData};

pub async fn creates_commands(http: &Arc<Http>, is_ok: bool) {
    if is_ok {
        delete_command(http).await;
    }
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
    let mut permission = Permissions::SEND_MESSAGES;
    if command.perm {
        let mut perm_bit: u64 = 0;
        let perm_list = command.default_permissions.clone().unwrap();
        for permi in perm_list {
            let permission: Permissions = permi.permission.into();
            perm_bit += permission.bits()
        }
        permission = Permissions::from_bits(perm_bit).unwrap()
    }

    let mut nsfw = command.nsfw.clone();

    if command.name.as_str() == "image" {
        let honor_nsfw = env::var("IMAGE_GENERATION_MODELS_ON").unwrap_or(String::from("fale"));
        let is_ok = honor_nsfw.to_lowercase() == "true";
        nsfw = is_ok
    }

    let mut build = CreateCommand::new(&command.name)
        .description(&command.desc)
        .dm_permission(command.dm_command)
        .nsfw(nsfw)
        .default_member_permissions(permission);
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

    if command.arg_num > (0u32) {
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
        options_build = options_build.set_autocomplete(option.autocomplete);

        options_builds.push(options_build)
    }

    options_builds
}

async fn delete_command(http: &Arc<Http>) {
    let cmds = Command::get_global_commands(http).await.unwrap();
    for cmd in cmds {
        trace!("Removing {:?}", cmd.name);
        let error = Command::delete_global_command(http, cmd.id).await;
        error!("{:?}", error);
    }
}
