use crate::register::function::common::{get_option, get_permission, get_vec};
use crate::register::structure::guild_command::GuildCommand;
use serenity::all::{CommandType, CreateCommand, GuildId, Http};
use std::error::Error;
use std::sync::Arc;
use tracing::{error, trace};

pub async fn creates_guild_commands(http: &Arc<Http>) {
    let commands = match get_commands("./json/guild_command") {
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

pub fn get_commands(path: &str) -> Result<Vec<GuildCommand>, Box<dyn Error>> {
    let commands: Vec<GuildCommand> = get_vec(path)?;

    if commands.is_empty() {
        trace!("No commands found in the directory: {:?}", path);
    }

    Ok(commands)
}

async fn create_command(command: &GuildCommand, http: &Arc<Http>) {
    let mut command_build = CreateCommand::new(&command.name)
        .nsfw(command.nsfw)
        .kind(CommandType::ChatInput)
        .description(&command.desc);

    command_build = get_permission(&command.permissions, command_build);

    command_build = match &command.args {
        Some(args) => {
            let options = get_option(args);

            command_build.set_options(options)
        }
        None => command_build,
    };

    if let Some(locale) = &command.localised {
        for locale in locale {
            command_build = command_build
                .name_localized(&locale.code, &locale.name)
                .description_localized(&locale.code, &locale.desc);
        }
    }

    let guild_id = GuildId::from(command.guild_id);

    let e = http.create_guild_command(guild_id, &command_build).await;

    match e {
        Ok(_) => (),
        Err(e) => {
            error!("Failed to create command: {:?}", e);
        }
    }
}
