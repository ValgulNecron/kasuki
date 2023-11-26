use crate::error_enum::AppError;
use crate::function::error_management::no_lang_error::{
    error_cant_read_langage_file, error_langage_file_not_found, error_no_langage_guild_id,
    error_parsing_langage_json,
};
use serenity::client::Context;
use serenity::model::prelude::application_command::ApplicationCommandInteraction;

pub async fn error_dispatching(
    error: AppError,
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) {
    match error {
        AppError::LocalisationFileError(..) => error_langage_file_not_found(ctx, command).await,
        AppError::LocalisationReadError(..) => error_cant_read_langage_file(ctx, command).await,
        AppError::LocalisationParsingError(..) => error_parsing_langage_json(ctx, command).await,
        AppError::LangageGuildIdError(..) => error_no_langage_guild_id(ctx, command).await,
        _ => {}
    }
}
