use crate::error_enum::AppError;
use crate::error_enum::AppError::*;
use crate::function::error_management::error_avatar::error_no_avatar;
use crate::function::error_management::error_getting_option::error_no_option;
use crate::function::error_management::error_module::{error_module_off, error_no_module};
use crate::function::error_management::error_no::error_no_user_specified;
use crate::function::error_management::no_lang_error::{
    error_cant_read_langage_file, error_langage_file_not_found, error_no_langage_guild_id,
    error_parsing_langage_json, no_langage_error,
};
use log::error;
use serenity::client::Context;
use serenity::futures::TryFutureExt;
use serenity::model::prelude::application_command::ApplicationCommandInteraction;

pub async fn error_dispatching(
    error: AppError,
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) {
    match error {
        OptionError(..) => error_no_option(ctx, command).await,
        CommandSendingError(..) => error!("Failed to respond to the command."),
        LocalisationFileError(..) => error_langage_file_not_found(ctx, command).await,
        LocalisationReadError(..) => error_cant_read_langage_file(ctx, command).await,
        LocalisationParsingError(..) => error_parsing_langage_json(ctx, command).await,
        LangageGuildIdError(..) => error_no_langage_guild_id(ctx, command).await,
        NoLangageError(..) => no_langage_error(ctx, command).await,
        FailedToGetUser(..) => error_no_user_specified(ctx, command).await,
        NoAvatarError(..) => error_no_avatar(ctx, command).await,
        NoCommandOption(..) => error!("Error command option."),
        SqlInsertError(..) => error!("Error Insert sql."),
        SqlSelectError(..) => error!("Error Select sql."),
        ModuleError(..) => error_no_module(ctx, command).await,
        ModuleOffError(..) => error_module_off(ctx, command).await,
        UnknownCommandError(..) => error!("What that shit you've done."),
        _ => {}
    }
}
