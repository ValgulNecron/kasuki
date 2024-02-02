#[derive(Debug, Clone)]
pub enum AppError {
    Error(Error),
    DifferedError(DifferedError),
    NotACommandError(NotACommandError),
    ComponentError(ComponentError),
    JoiningError(JoiningError),
}

#[derive(Debug, Clone)]
pub enum DifferedError {
    CreatingImageError(String),
    FileExtensionError(String),
    CopyBytesError(String),
    GettingBytesError(String),
    TokenError(String),
    ImageModelError(String),
    HeaderError(String),
    ResponseError(String),
    FailedUrlError(String),
    DifferedOptionError(String),
    FailedToGetBytes(String),
    WritingFile(String),
    DifferedCommandSendingError(String),
    NotAiringError(String),
    MediaError(String),
    CreatingWebhookError(String),
    NoStatisticError(String),
    FailedToUploadImage(String),
    FailedToCreateFolder(String),
    FailedToWriteFile(String),
    FailedToGetImage(String),
}

#[derive(Debug, Clone)]
pub enum Error {
    NotNSFWError(String),
    NotAValidUrlError(String),
    NotAValidGameError(String),
    ErrorOptionError(String),
    ErrorCommandSendingError(String),
    LocalisationFileError(String),
    LocalisationReadError(String),
    LocalisationParsingError(String),
    NoLangageError(String),
    FailedToGetUser(String),
    NoCommandOption(String),
    SqlInsertError(String),
    ModuleError(String),
    ModuleOffError(String),
    UnknownCommandError(String),
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
    SetLoggerError(String),
    GettingDatabaseFileError(String),
    CreatingDatabaseFileError(String),
    CreatingDatabaseError(String),
    InsertingDatabaseError(String),
    CreatingTableError(String),
    CreatingPoolError(String),
    FailedToUpdateDatabase(String),
    SqlSelectError(String),
}

#[derive(Debug, Clone)]
pub enum ComponentError {
    ComponentOptionError(String),
    SendingError(String),
}

#[derive(Debug, Clone)]
pub enum JoiningError {
    FailedToCreateDirectory(String),
}
