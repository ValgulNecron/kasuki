use crate::command_register::command_struct::common::{Arg, Choice, ChoiceLocalised, Localised};
use crate::command_register::command_struct::subcommand::Command;
use crate::command_register::command_struct::subcommand_group::SubCommand;
use serenity::all::{CommandOptionType, CreateCommandOption};

pub fn get_option(args: &Vec<Arg>) -> Vec<CreateCommandOption> {
    let mut options = Vec::new();
    for arg in args {
        let mut option =
            CreateCommandOption::new(CommandOptionType::from(arg.arg_type), &arg.name, &arg.desc)
                .required(arg.required)
                .set_autocomplete(arg.autocomplete);

        option = match &arg.choices {
            Some(choices) => add_choices(option, choices),
            None => option,
        };

        option = match &arg.localised {
            Some(localised) => add_localised(option, localised),
            None => option,
        };

        options.push(option);
    }
    options
}

fn add_localised(mut option: CreateCommandOption, locales: &Vec<Localised>) -> CreateCommandOption {
    for locale in locales {
        option = option
            .name_localized(&locale.code, &locale.name)
            .description_localized(&locale.code, &locale.desc);
    }
    option
}

fn add_choices(mut option: CreateCommandOption, choices: &Vec<Choice>) -> CreateCommandOption {
    for choice in choices {
        option = option.add_string_choice(&choice.option_choice, &choice.option_choice);
        option = match &choice.option_choice_localised {
            Some(localised) => add_choices_localised(option, localised, &choice.option_choice),
            None => option,
        }
    }
    option
}

fn add_choices_localised(
    option: CreateCommandOption,
    locales: &Vec<ChoiceLocalised>,
    name: &String,
) -> CreateCommandOption {
    let vec = locales
        .iter()
        .map(|locale| (&locale.code, &locale.name))
        .collect::<Vec<_>>();
    option.add_string_choice_localized(name, name, vec)
}

pub fn get_subcommand_option(commands: &Vec<Command>) -> Vec<CreateCommandOption> {
    let mut options = Vec::new();
    for command in commands {
        let mut option =
            CreateCommandOption::new(CommandOptionType::SubCommand, &command.name, &command.desc);
        option = match &command.args {
            Some(args) => {
                let options = get_option(args);
                option.set_sub_options(options)
            }
            None => option,
        };
        option = match &command.localised {
            Some(localised) => add_localised(option, localised),
            None => option,
        };
        options.push(option);
    }
    options
}

pub fn get_subcommand_group_option(subcommands: &Vec<SubCommand>) -> Vec<CreateCommandOption> {
    let mut options = Vec::new();
    for subcommand in subcommands {
        let mut option = CreateCommandOption::new(
            CommandOptionType::SubCommand,
            &subcommand.name,
            &subcommand.desc,
        );
        option = match &subcommand.command {
            Some(command) => {
                let options = get_subcommand_option(command);
                option.set_sub_options(options)
            }
            None => option,
        };
        option = match &subcommand.localised {
            Some(localised) => add_localised(option, localised),
            None => option,
        };
        options.push(option);
    }
    options
}
