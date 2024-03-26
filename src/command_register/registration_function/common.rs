use crate::command_register::command_struct::common::{Arg, Choice, ChoiceLocalised, Localised};
use serenity::all::{CommandOptionType, CreateCommandOption};

pub fn get_option(args: Vec<Arg>) -> Vec<CreateCommandOption> {
    let mut options = Vec::new();
    for arg in args {
        let mut option =
            CreateCommandOption::new(CommandOptionType::from(arg.arg_type), arg.name, arg.desc)
                .required(arg.required);
        option = match arg.choices {
            Some(choices) => add_choices(option, choices),
            None => option,
        }
    }
    options
}

fn add_choices(mut option: CreateCommandOption, choices: Vec<Choice>) -> CreateCommandOption {
    for choice in choices {
        option = option.add_string_choice(&choice.option_choice, &choice.option_choice);
        option = match choice.option_choice_localised {
            Some(localised) => add_choices_localised(option, localised, &choice.option_choice),
            None => option,
        }
    }
    option
}

fn add_choices_localised(
    mut option: CreateCommandOption,
    locales: Vec<ChoiceLocalised>,
    name: &String,
) -> CreateCommandOption {
    for locale in locales {
        option = option.add_string_choice_localized(&name, &name, (&locale.code, &locale.name));
    }
    option
}
