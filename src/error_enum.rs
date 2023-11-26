use once_cell::sync::Lazy;

#[derive(Debug, Clone)]
pub enum AppError {
    CommonError(String),
    OptionError(String),
    CommandSendingError(String),
    LocalisationFileError(String),
    LocalisationReadError(String),
    LocalisationParsingError(String),
    LangageGuildIdError(String),
}

pub static OPTION_ERROR: Lazy<AppError> =
    Lazy::new(|| AppError::OptionError(String::from("The option contain no value")));
pub static COMMAND_SENDING_ERROR: Lazy<AppError> = Lazy::new(|| {
    AppError::CommandSendingError(String::from(
        "Error while sending the response of the command.",
    ))
});
