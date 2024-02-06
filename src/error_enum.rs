#[derive(Debug, Clone)]
pub enum AppError {
    Error(CommandError),
    DifferedError(DifferedCommandError),
    NotACommandError(NotACommandError),
    ComponentError(ComponentError),
    JoiningError(JoiningError),
    NewMemberError(NewMemberError),
}

#[derive(Debug, Clone)]
pub enum DifferedCommandError {
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
pub enum CommandError {
    NotNSFWError(String),
    NotAValidUrlError(String),
    NotAValidGameError(String),
    ErrorOptionError(String),
    ErrorCommandSendingError(String),
    LocalisationFileError(String),
    LocalisationReadError(String),
    LocalisationParsingError(String),
    NoLanguageError(String),
    FailedToGetUser(String),
    NoCommandOption(String),
    SqlInsertError(String),
    ModuleError(String),
    ModuleOffError(String),
    UnknownError(String),
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

#[derive(Debug, Clone)]
pub enum NewMemberError {
    NewMemberErrorOptionError(String),
    NewMemberLocalisationParsingError(String),
    NewMemberLocalisationReadError(String),
NewMemberLocalisationFileError(String),
    NewMemberNoLanguageError(String),
}
