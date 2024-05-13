use serenity::all::{CommandOptionType, CreateCommand, CreateCommandOption, Permissions};
use std::fs;

use crate::command_register::command_struct::common::{
    Arg, Choice, ChoiceLocalised, DefaultPermission, Localised,
};
use crate::command_register::command_struct::subcommand::Command;
use crate::command_register::command_struct::subcommand_group::SubCommand;
use crate::helper::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use crate::helper::read_file::read_file_as_string;

/// This function takes a vector of `Arg` structs and returns a vector of `CreateCommandOption` structs.
/// Each `Arg` struct is converted into a `CreateCommandOption` with the `CommandOptionType` from the `Arg` type.
/// If the `Arg` has choices, they are converted into `CreateCommandOption` structs and set as sub-options.
/// If the `Arg` has localised versions, they are added to the `CreateCommandOption`.
///
/// # Arguments
///
/// * `args` - A vector of `Arg` structs.
///
/// # Returns
///
/// A vector of `CreateCommandOption` structs.
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

/// This function takes a `CreateCommandOption` and a vector of `Localised` structs and returns a `CreateCommandOption` with the localised versions added.
///
/// # Arguments
///
/// * `option` - A `CreateCommandOption` struct.
/// * `locales` - A vector of `Localised` structs.
///
/// # Returns
///
/// A `CreateCommandOption` struct with the localised versions added.
fn add_localised(mut option: CreateCommandOption, locales: &Vec<Localised>) -> CreateCommandOption {
    for locale in locales {
        option = option
            .name_localized(&locale.code, &locale.name)
            .description_localized(&locale.code, &locale.desc);
    }
    option
}

/// This function takes a `CreateCommandOption` and a vector of `Choice` structs and returns a `CreateCommandOption` with the choices added.
///
/// # Arguments
///
/// * `option` - A `CreateCommandOption` struct.
/// * `choices` - A vector of `Choice` structs.
///
/// # Returns
///
/// A `CreateCommandOption` struct with the choices added.
fn add_choices(mut option: CreateCommandOption, choices: &Vec<Choice>) -> CreateCommandOption {
    for choice in choices {
        option = match &choice.option_choice_localised {
            Some(localised) => add_choices_localised(option, localised, &choice.option_choice),
            None => option.add_string_choice(&choice.option_choice, &choice.option_choice),
        }
    }
    option
}

/// This function takes a `CreateCommandOption`, a slice of `ChoiceLocalised` structs, and a `String` and returns a `CreateCommandOption` with the localised choices added.
///
/// # Arguments
///
/// * `option` - A `CreateCommandOption` struct.
/// * `locales` - A slice of `ChoiceLocalised` structs.
/// * `name` - A `String` representing the name of the choice.
///
/// # Returns
///
/// A `CreateCommandOption` struct with the localised choices added.
fn add_choices_localised(
    option: CreateCommandOption,
    locales: &[ChoiceLocalised],
    name: &String,
) -> CreateCommandOption {
    let vec = locales
        .iter()
        .map(|locale| (&locale.code, &locale.name))
        .collect::<Vec<_>>();
    option.add_string_choice_localized(name, name, vec)
}

/// This function takes a vector of `Command` structs and returns a vector of `CreateCommandOption` structs.
/// Each `Command` struct is converted into a `CreateCommandOption` with the `CommandOptionType::SubCommand` type.
/// If the `Command` has arguments, they are converted into `CreateCommandOption` structs and set as sub-options.
/// If the `Command` has localised versions, they are added to the `CreateCommandOption`.
///
/// # Arguments
///
/// * `commands` - A vector of `Command` structs.
///
/// # Returns
///
/// A vector of `CreateCommandOption` structs.
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

/// This function takes a vector of `SubCommand` structs and returns a vector of `CreateCommandOption` structs.
/// Each `SubCommand` struct is converted into a `CreateCommandOption` with the `CommandOptionType::SubCommandGroup` type.
/// If the `SubCommand` has commands, they are converted into `CreateCommandOption` structs and set as sub-options.
/// If the `SubCommand` has localised versions, they are added to the `CreateCommandOption`.
///
/// # Arguments
///
/// * `subcommands` - A vector of `SubCommand` structs.
///
/// # Returns
///
/// A vector of `CreateCommandOption` structs.
pub fn get_subcommand_group_option(subcommands: &Vec<SubCommand>) -> Vec<CreateCommandOption> {
    let mut options = Vec::new();
    for subcommand in subcommands {
        let mut option = CreateCommandOption::new(
            CommandOptionType::SubCommandGroup,
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

/// This function takes an `Option` containing a vector of `DefaultPermission` structs and a `CreateCommand` struct,
/// and returns a `CreateCommand` with the default member permissions set.
///
/// If the `Option` is `Some`, it iterates over the `DefaultPermission` structs, converts each one into a `Permissions` struct,
/// and combines them using bitwise OR. The combined permissions are then set as the default member permissions of the `CreateCommand`.
///
/// If the `Option` is `None`, it returns the `CreateCommand` without modifying it.
///
/// # Arguments
///
/// * `permissions` - An `Option` containing a vector of `DefaultPermission` structs.
/// * `command_build` - A `CreateCommand` struct.
///
/// # Returns
///
/// A `CreateCommand` struct with the default member permissions set.
pub fn get_permission(
    permissions: &Option<Vec<DefaultPermission>>,
    mut command_build: CreateCommand,
) -> CreateCommand {
    command_build = match permissions {
        Some(permissions) => {
            let mut perm_bit: u64 = 0;
            for perm in permissions {
                let permission: Permissions = perm.permission.into();
                perm_bit |= permission.bits()
            }
            let permission = Permissions::from_bits(perm_bit).unwrap();
            command_build.default_member_permissions(permission)
        }
        None => command_build,
    };
    command_build
}

pub fn get_vec<T: serde::Deserialize<'static> + Clone>(path: &str) -> Result<Vec<T>, AppError> {
    let mut commands: Vec<T> = Vec::new();
    let paths = fs::read_dir(path).map_err(|e| AppError {
        message: format!("Failed to read directory: {:?} with error {}", path, e),
        error_type: ErrorType::File,
        error_response_type: ErrorResponseType::None,
    })?;
    for entry in paths {
        let entry = entry.map_err(|e| AppError {
            message: format!("Failed to read path with error {}", e),
            error_type: ErrorType::File,
            error_response_type: ErrorResponseType::None,
        })?;
        let path = entry.path();
        if path.is_file() && path.extension().unwrap_or_default() == "json" {
            // Read the file content once and store it in a variable outside the loop
            let json_content: String = read_file_as_string(path.to_str().unwrap())?;
            // Convert the String to a &'static str by cloning it into a static buffer
            let json: &'static str = Box::leak(json_content.into_boxed_str());
            // Now, `json` has a 'static lifetime and can be passed to `serde_json::from_str`
            let command: T = serde_json::from_str(json).map_err(|e| AppError {
                message: format!(
                    "Failed to parse file: {:?} with error {}",
                    path.as_path(),
                    e
                ),
                error_type: ErrorType::File,
                error_response_type: ErrorResponseType::None,
            })?;
            commands.push(command);
        }
    }
    Ok(commands)
}
