#[derive(Debug, Clone)]
pub enum AppError {
    OptionError(String),
    CommandSendingError(String),
    LocalisationFileError(String),
    LocalisationReadError(String),
    LocalisationParsingError(String),
    LangageGuildIdError(String),
    NoLangageError(String),
    FailedToGetUser(String),
    NoAvatarError(String),
    NoCommandOption(String),
    SqlInsertError(String),
    SqlSelectError(String),
    ModuleError(String),
    ModuleOffError(String),
    UnknownCommandError(String),
    NoAnimeError(String),
    NoAnimeDifferedError(String),
    NoMediaDifferedError(String),
    CreatingWebhookDifferedError(String),
}
