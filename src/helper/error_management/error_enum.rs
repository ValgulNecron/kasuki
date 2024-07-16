
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Clone)]
pub enum Error {
    #[error("Error while getting guild data: {0}")]
    GettingGuild(String),
    #[error("Error while getting an option: {0}")]
    Option(String),
    #[error("Error while processing image: {0}")]
    ImageProcessing(String),
    #[error("Error while doing a web request: {0}")]
    WebRequest(String),
    #[error("Error while getting a byte: {0}")]
    Byte(String),
    #[error("Error while doing a webhook request: {0}")]
    Webhook(String),
    #[error("Error with the database: {0}")]
    Database(String),
    #[error("Error while sending the response: {0}")]
    Sending(String),
    #[error("Error while initializing the logger: {0}")]
    Logger(String),
}

#[derive(Debug, Error, PartialEq, Clone)]
pub enum ResponseError {
    #[error("Error while sending the response: {0}")]
    Sending(String),
    #[error("Error while getting an option: {0}")]
    Option(String),
    #[error("Error with a file: {0}")]
    File(String),
    #[error("Error while doing a web request: {0}")]
    WebRequest(String),
    #[error("Error while decoding: {0}")]
    Decoding(String),
    #[error("Error with getting a full user or guild: {0}")]
    UserOrGuild(String),
    #[error("Error with a json: {0}")]
    Json(String),
    #[error("The media is adult and the channel is not adult only")]
    AdultMedia,
}

#[derive(Debug, Error, PartialEq, Clone)]
pub enum FollowupError {
    #[error("Error while sending the followup: {0}")]
    Sending(String),
    #[error("Error while getting an option: {0}")]
    Option(String),
    #[error("Error while processing image: {0}")]
    ImageProcessing(String),
    #[error("Error while getting a byte: {0}")]
    Byte(String),
    #[error("Error with a webhook: {0}")]
    Webhook(String),
    #[error("Error while decoding: {0}")]
    Decoding(String),
    #[error("Error with a file: {0}")]
    File(String),
    #[error("Error while doing a web request: {0}")]
    WebRequest(String),
    #[error("Error with a json: {0}")]
    Json(String),
    #[error("Error with getting a full user or guild: {0}")]
    UserOrGuild(String),
}

#[derive(Debug, Error, PartialEq, Clone)]
pub enum UnknownResponseError {
    #[error("Error while parsing data: {0}")]
    Parsing(String),
    #[error("Error while getting an option: {0}")]
    Option(String),
    #[error("Error with a file: {0}")]
    File(String),
    #[error("Error while doing a web request: {0}")]
    WebRequest(String),
    #[error("Error with a json: {0}")]
    Json(String),
    #[error("Error while sending the followup: {0}")]
    Sending(String),
    #[error("Error with getting a full user or guild: {0}")]
    UserOrGuild(String),
    #[error("Error with the database: {0}")]
    Database(String),
}
