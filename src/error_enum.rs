#[derive(Debug, Clone)]
pub enum AppError {
    Error(Error),
    DifferedError(DifferedError),
    NotACommandError(NotACommandError),
}

#[derive(Debug, Clone)]
pub enum DifferedError {
    DifferedFailedToCreateDirectory(String),
    DifferedCreatingImageError(String),
    DifferedFileTypeError(String),
    DifferedFileExtensionError(String),
    DifferedCopyBytesError(String),
    DifferedGettingBytesError(String),
    DifferedTokenError(String),
    DifferedImageModelError(String),
    DifferedHeaderError(String),
    DifferedResponseError(String),
    DifferedFailedUrlError(String),
    DifferedOptionError(String),
    DifferedFailedToGetBytes(String),
    DifferedWritingFile(String),
    DifferedCommandSendingError(String),
    DifferedNotAiringError(String),
    DifferedMediaError(String),
    DifferedCreatingWebhookError(String),
    DifferedNoStatisticError(String),
}

#[derive(Debug, Clone)]
pub enum Error {
    NotNSFWError(String),
    NotAValidUrlError(String),
    NotAValidGameError(String),
    ErrorGettingUserList(String),
    CreatingImageError(String),
    DecodingImageError(String),
    FailedToGetImage(String),
    FailedToCreateFolder(String),
    FailedToUploadImage(String),
    FailedToWriteFile(String),
    FailedToUpdateDatabase(String),
    FailedToCreateDirectory(String),
    OptionError(String),
    CommandSendingError(String),
    LocalisationFileError(String),
    LocalisationReadError(String),
    LocalisationParsingError(String),
    NoLangageError(String),
    FailedToGetUser(String),
    NoCommandOption(String),
    SqlInsertError(String),
    SqlSelectError(String),
    SqlCreateError(String),
    ModuleError(String),
    ModuleOffError(String),
    UnknownCommandError(String),
    FailedToCreateAFile(String),
    MediaGettingError(String),
    UserGettingError(String),
    NotAValidTypeError(String),
    CharacterGettingError(String),
    StaffGettingError(String),
    StudioGettingError(String),
    FileTypeError(String),
}

#[derive(Debug, Clone)]
pub enum NotACommandError {
    NotACommandOptionError(String),
    NotACommandSetLoggerError(String),
    NotACommandGettingDatabaseFileError(String),
    NotACommandCreatingDatabaseFileError(String),
    NotACommandCreatingDatabaseError(String),
    NotACommandInsertingDatabaseError(String),
    NotACommandCreatingTableError(String),
    NotACommandCreatingPoolError(String),
}
