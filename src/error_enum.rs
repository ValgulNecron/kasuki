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
    NoLangageError(String),
    FailedToGetUser(String),
    NoAvatarError(String),
    NoBannerError(String),
}

pub static OPTION_ERROR: Lazy<AppError> =
    Lazy::new(|| AppError::OptionError(String::from("The option contain no value")));
pub static COMMAND_SENDING_ERROR: Lazy<AppError> = Lazy::new(|| {
    AppError::CommandSendingError(String::from(
        "Error while sending the response of the command.",
    ))
});

pub static NO_AVATAR_ERROR: Lazy<AppError> =
    Lazy::new(|| AppError::NoAvatarError(String::from("Error while getting the user avatar.")));

pub static NO_BANNER_ERROR: Lazy<AppError> =
    Lazy::new(|| AppError::NoBannerError(String::from("Error while getting the user banner.")));
