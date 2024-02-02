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
    FileTypeError(String),
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
    DecodingImageError(String),
    FailedToCreateFolder(String),
    FailedToWriteFile(String),
    FailedToUpdateDatabase(String),
    FailedToCreateDirectory(String),
    ErrorOptionError(String),
    ErrorCommandSendingError(String),
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
    SetLoggerError(String),
    GettingDatabaseFileError(String),
    CreatingDatabaseFileError(String),
    CreatingDatabaseError(String),
    InsertingDatabaseError(String),
    CreatingTableError(String),
    CreatingPoolError(String),
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
