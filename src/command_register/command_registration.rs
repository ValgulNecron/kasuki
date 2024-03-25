use std::fs;
use std::sync::Arc;

use serenity::all::{
    Command, CommandOptionType, CreateCommand, CreateCommandOption, Http, Permissions,
};
use tracing::{error, info, trace};

use crate::command_register::command_structure::{
    get_commands, Arg, CommandData, Localised, LocalisedArg, RemoteCommandOptionType,
    SubCommandData,
};

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
    info!("Started creating command");
    for command in commands {
        trace!("Creating: {}", command.name);
        create_command(&command, http).await;
    }
    info!("Done creating command");
}

async fn create_command(command: &CommandData, http: &Arc<Http>) {
    let mut build = CreateCommand::new(&command.name)
        .description(&command.desc)
        .dm_permission(command.dm_command)
        .nsfw(command.nsfw);
    let mut permission = Permissions::SEND_MESSAGES;
    if command.perm {
        let mut perm_bit: u64 = 0;
        let perm_list = command.default_permissions.clone().unwrap();
        for perm in perm_list {
            let permission: Permissions = perm.permission.into();
            perm_bit |= permission.bits()
        }
        permission = Permissions::from_bits(perm_bit).unwrap()
    }
    build = build.default_member_permissions(permission);

    build = add_localised_command(build, &command.localised).await;

    if command.arg_num > 0 {
        let options = get_options(&command.args).await;
        for option in options {
            build = build.add_option(option);
        }
    }

    match Command::create_global_command(http, build).await {
        Ok(_) => {}
        Err(e) => {
            error!("{} for command {}", e, command.name);
        }
    }
}

async fn get_options(args: &Option<Vec<Arg>>) -> Vec<CreateCommandOption> {
    let args = args.clone().unwrap();
    let mut options: Vec<CreateCommandOption> = Vec::new();
    for arg in args {
        if arg.command_type == RemoteCommandOptionType::SubCommand {
            options.insert(options.len(), create_subcommand_option(arg).await);
        } else if arg.command_type == RemoteCommandOptionType::SubCommandGroup {
            options.insert(options.len(), create_subcommand_group_option(arg).await);
        } else {
            options.insert(options.len(), create_option(arg).await)
        }
    }
    options
}

async fn create_subcommand_option(arg: Arg) -> CreateCommandOption {
    let path = format!("./json/command/{}", &arg.file.unwrap());
    let content = fs::read_to_string(&path).unwrap_or_default();
    let subcommand: SubCommandData = serde_json::from_str(&content).unwrap();
    let mut subcommand_option = CreateCommandOption::new(
        CommandOptionType::from(arg.command_type),
        subcommand.name,
        subcommand.desc,
    );
    if subcommand.arg_num > 0 {
        for arg in &subcommand.args.unwrap() {
            let option = create_option(arg.clone()).await;
            subcommand_option = subcommand_option.add_sub_option(option)
        }
    }

    subcommand_option = add_localised_option(subcommand_option, arg.localised_args).await;

    subcommand_option
}

async fn create_option(arg: Arg) -> CreateCommandOption {
    let mut option = CreateCommandOption::new(
        CommandOptionType::from(arg.command_type),
        arg.name,
        arg.desc,
    )
    .set_autocomplete(arg.autocomplete)
    .required(arg.required);

    if let Some(choices) = arg.choices {
        for choice in choices {
            option = option.add_string_choice(&choice.option_choice, &choice.option_choice);
        }
    }

    option = add_localised_option(option, arg.localised_args).await;

    option
}

async fn create_subcommand_group_option(arg: Arg) -> CreateCommandOption {
    let path = format!("./json/command/{}", &arg.file.unwrap());
    let subcommand: SubCommandData = serde_json::from_str(path.as_str()).unwrap();
    let mut subcommand_option = CreateCommandOption::new(
        CommandOptionType::from(arg.command_type),
        subcommand.name,
        subcommand.desc,
    );
    if subcommand.arg_num > 0 {
        for arg in &subcommand.args.unwrap() {
            if arg.command_type == RemoteCommandOptionType::SubCommand {
                subcommand_option =
                    subcommand_option.add_sub_option(create_subcommand_option(arg.clone()).await);
            } else {
                subcommand_option =
                    subcommand_option.add_sub_option(create_option(arg.clone()).await)
            }
        }
    }

    subcommand_option = add_localised_option(subcommand_option, arg.localised_args).await;

    subcommand_option
}

async fn add_localised_option(
    mut command_option: CreateCommandOption,
    localised: Option<Vec<LocalisedArg>>,
) -> CreateCommandOption {
    match localised {
        Some(locals) => {
            for local in locals {
                command_option = command_option
                    .name_localized(&local.code, &local.name)
                    .description_localized(&local.code, &local.desc)
            }
            command_option
        }
        None => command_option,
    }
}

async fn add_localised_command(
    mut command: CreateCommand,
    localised: &Option<Vec<Localised>>,
) -> CreateCommand {
    match localised {
        Some(locals) => {
            for local in locals {
                command = command
                    .name_localized(&local.code, &local.name)
                    .description_localized(&local.code, &local.desc)
            }
            command
        }
        None => command,
    }
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
