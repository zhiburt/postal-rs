use thiserror::Error;

/// PostalError denotes crate erorrs
#[derive(Error, Debug)]
pub enum PostalError {
    #[error("data store disconnected")]
    Network(#[from] reqwest::Error),
    #[error("data store disconnected")]
    UrlIssue(#[from] url::ParseError),
    #[error("send error({code:?}): {message:?}")]
    Error { code: String, message: String },
    #[error("internal error on postal side")]
    InternalServerError,
    #[error("postal server unavailable")]
    ServiceUnavailableError,
    #[error("Request should likely be sent to an another URL")]
    ExpectedAlternativeUrl,
}
