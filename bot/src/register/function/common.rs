use anyhow::{Context, Result};
use std::borrow::Cow;
use std::collections::HashMap;
use std::fs;

use serenity::all::{
    CommandOptionType, CreateCommand, CreateCommandOption, InstallationContext, InteractionContext,
    Permissions,
};
use tracing::trace;

use crate::helper::read_file::read_file_as_string;
use crate::register::structure::common::{
    Arg, Choice, ChoiceLocalised, CommandInstallationContext, CommandIntegrationContext,
    DefaultPermission, Localised,
};
use crate::register::structure::subcommand::Command;
use crate::register::structure::subcommand_group::SubCommand;

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

fn add_localised<'a>(
    mut option: CreateCommandOption<'a>,
    locales: &'a Vec<Localised>,
) -> CreateCommandOption<'a> {
    for locale in locales {
        option = option
            .name_localized(&locale.code, &locale.name)
            .description_localized(&locale.code, &locale.desc);
    }

    option
}

fn add_choices<'a>(
    mut option: CreateCommandOption<'a>,
    choices: &'a Vec<Choice>,
) -> CreateCommandOption<'a> {
    for choice in choices {
        option = match &choice.option_choice_localised {
            Some(localised) => add_choices_localised(option, localised, &choice.option_choice),
            None => option.add_string_choice(&choice.option_choice, &choice.option_choice),
        }
    }

    option
}

fn add_choices_localised<'a>(
    option: CreateCommandOption<'a>,
    locales: &'a [ChoiceLocalised],
    name: &'a String,
) -> CreateCommandOption<'a> {
    let vec = locales
        .iter()
        .map(|locale| {
            (
                Cow::from(locale.code.as_str()),
                Cow::from(locale.name.as_str()),
            )
        })
        .collect::<HashMap<Cow<'a, str>, Cow<'a, str>>>();

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

pub fn get_permission<'a>(
    permissions: &'a Option<Vec<DefaultPermission>>,
    mut command_build: CreateCommand<'a>,
) -> CreateCommand<'a> {
    command_build = match permissions {
        Some(permissions) => {
            let mut perm_bit: u64 = 0;

            for perm in permissions {
                let permission: Permissions = perm.permission.into();

                perm_bit |= permission.bits()
            }

            trace!("{:?}", perm_bit);

            let permission = Permissions::from_bits(perm_bit).unwrap_or(Permissions::empty());

            command_build.default_member_permissions(permission)
        }
        None => command_build,
    };

    command_build
}

pub fn get_vec<T: serde::Deserialize<'static> + Clone>(path: &str) -> Result<Vec<T>> {
    let mut commands: Vec<T> = Vec::new();

    let paths = fs::read_dir(path).context(format!("Failed to read dir {}", path))?;

    for entry in paths {
        let start = std::time::Instant::now();

        let entry = entry.context("Failed to get an entry")?;

        let path = entry.path();

        if path.is_file() && path.extension().unwrap_or_default() == "json" {
            // Read the file content once and store it in a variable outside the loop
            let json_content: String = read_file_as_string(path.to_string_lossy().as_ref())
                .context(format!("failed to read the file {:?}", path))?;

            let json: &'static str = Box::leak(json_content.into_boxed_str());

            // Now, `json` has a 'static lifetime and can be passed to `serde_json::from_str`
            let command: T = serde_json::from_str(&json)
                .context("failed to create the command struct from the json")?;

            commands.push(command);
        }

        let duration = start.elapsed();

        trace!("Time taken to parse command structure: {:?}", duration);
    }

    Ok(commands)
}

pub fn get_vec_integration_context(context: &CommandIntegrationContext) -> Vec<InteractionContext> {
    let mut contexts = Vec::new();

    if context.guild {
        contexts.push(InteractionContext::Guild);
    }

    if context.bot_dm {
        contexts.push(InteractionContext::BotDm);
    }

    if context.private_channel {
        contexts.push(InteractionContext::PrivateChannel);
    }

    contexts
}

pub fn get_vec_installation_context(
    context: &CommandInstallationContext,
) -> Vec<InstallationContext> {
    let mut contexts = Vec::new();

    if context.guild {
        contexts.push(InstallationContext::Guild);
    }

    if context.user {
        contexts.push(InstallationContext::User);
    }

    contexts
}
