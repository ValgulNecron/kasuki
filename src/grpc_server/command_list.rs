use tracing::error;
use crate::command_register::command_struct::{command, common, subcommand, subcommand_group};
use crate::command_register::command_struct::common::Choice;
use crate::command_register::registration_function::register_command::get_commands;
use crate::command_register::registration_function::register_subcommand::get_subcommands;
use crate::command_register::registration_function::register_subcommand_group::get_subcommands_group;
use crate::constant::BOT_COMMANDS;

#[derive(Clone)]
pub enum CommandItem {
    Command(Command),
    Subcommand(SubCommand),
    SubcommandGroup(SubCommandGroup),
}
#[derive(Clone)]
pub struct Command {
    pub name: String,
    pub desc: String,
    pub args: Vec<Arg>,
}
#[derive(Clone)]
pub struct SubCommand {
    pub name: String,
    pub desc: String,
    pub commands: Vec<Command>,
}
#[derive(Clone)]
pub struct SubCommandGroup {
    pub name: String,
    pub desc: String,
    pub commands: Vec<Command>,
    pub subcommands: Vec<SubCommand>,
}
#[derive(Clone)]
pub struct Arg {
    pub name: String,
    pub desc: String,
    pub required: bool,
    pub choices: Vec<String>,
}

pub fn get_command_list() {
    let mut commands_item = Vec::new();
    let commands = match get_commands("./json/command") {
        Err(e) => {
            error!("{:?}", e);
            return;
        }
        Ok(c) => c,
    };
    let subcommands = match get_subcommands("./json/subcommand") {
        Err(e) => {
            error!("{:?}", e);
            return;
        }
        Ok(c) => c,
    };
    let subcommands_group = match get_subcommands_group("./json/subcommand_group") {
        Err(e) => {
            error!("{:?}", e);
            return;
        }
        Ok(c) => c,
    };

    for command in commands {
        let com = create_com(command);
        commands_item.push(CommandItem::Command(com));
    }
    for subcommand in subcommands {
        let subcom = create_subcom(subcommand);
        commands_item.push(CommandItem::Subcommand(subcom));
    }
    for subcommand_group in subcommands_group {
        let subcom_group = create_subcom_group(subcommand_group);
        commands_item.push(CommandItem::SubcommandGroup(subcom_group));
    }

    unsafe {
        *BOT_COMMANDS = commands_item
    }
}

fn create_com(command: command::Command) -> Command {
    let mut args = Vec::new();
    if let Some(command_args) = command.args {
        args = create_arg(command_args);
    }
    let com = Command {
        name: command.name,
        desc: command.desc,
        args,
    };
    com
}

fn create_arg(command_args: Vec<common::Arg>) -> Vec<Arg> {
    let mut args = Vec::new();
    for arg in command_args {
        let mut choices_list = Vec::new();
        if let Some(choices) = arg.choices {
            choices_list = create_choice(choices);
        }
        let arg = Arg {
            name: arg.name,
            desc: arg.desc,
            required: arg.required,
            choices:choices_list,
        };
        args.push(arg);
    }
    args
}

fn create_choice(choices: Vec<Choice>) -> Vec<String> {
    let mut choices_list = Vec::new();
    for choice in choices {
        choices_list.push(choice.option_choice);
    }
    choices_list
}

fn create_subcom_com(command: subcommand::Command) -> Command {
    let mut args = Vec::new();
    if let Some(command_args) = command.args {
        args = create_arg(command_args);
    }
    let com = Command {
        name: command.name,
        desc: command.desc,
        args,
    };
    com
}

fn create_subcom(subcommand: subcommand::SubCommand) -> SubCommand {
    let mut commands = Vec::new();
    for command in subcommand.command.unwrap_or_default() {
        let com = create_subcom_com(command);
        commands.push(com);
    }
    let subcom = SubCommand {
        name: subcommand.name,
        desc: subcommand.desc,
        commands
    };
    subcom
}


fn create_subcom_group_subcom(subcommand: subcommand_group::SubCommand) -> SubCommand {
    let mut commands = Vec::new();
    for command in subcommand.command.unwrap_or_default() {
        let com = create_subcom_com(command);
        commands.push(com);
    }
    let subcom = SubCommand {
        name: subcommand.name,
        desc: subcommand.desc,
        commands
    };
    subcom
}
fn create_subcom_group(sub_command_group: subcommand_group::SubCommandGroup) -> SubCommandGroup {
    let mut commands = Vec::new();
    for command in sub_command_group.command.unwrap_or_default() {
        let com = create_subcom_com(command);
        commands.push(com);
    }
    let mut subcommands = Vec::new();
    for subcommand in sub_command_group.subcommands.unwrap_or_default() {
        let com = create_subcom_group_subcom(subcommand);
        subcommands.push(com);
    }
    let subcom = SubCommandGroup {
        name: sub_command_group.name,
        desc: sub_command_group.desc,
        commands,
        subcommands,
    };
    subcom
}