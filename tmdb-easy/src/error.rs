use std::fmt::Display;

use reqwest::StatusCode;

#[derive(Debug)]
pub struct Error {
    pub source: &'static str,
    pub error: ErrorKind,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Error in {}: {}", self.source, self.error)
    }
}

impl std::error::Error for Error {}

#[derive(Debug, thiserror::Error)]
pub enum ErrorKind {
    #[error("Network error: StatusCode {0}: {1}")]
    NetworkError(StatusCode, String),
    #[error("Problem authenticating, check your api key")]
    AuthenticationError,
    #[error(
        "Problem parsing answer from api, update dependency.\n\nOriginal Error: {0}\nSource text: {1}"
    )]
    ParsingError(String, String),
    #[error("Problem reading image data")]
    DecodingError,
    #[error("Failed creating a valid reqwest Request: {0}")]
    RequestCreationError(reqwest::Error),
}

impl From<tmdb_easy_raw::Error> for Error {
    fn from(value: tmdb_easy_raw::Error) -> Self {
        let source = value.context.source;
        Self {
            source,
            error: value.into(),
        }
    }
}

impl From<tmdb_easy_raw::Error> for ErrorKind {
    fn from(value: tmdb_easy_raw::Error) -> Self {
        if let Some(text) = value.context.text {
            Self::ParsingError(value.kind.to_string(), text)
        } else if let Some(status_code) = value.context.status {
            if status_code == StatusCode::UNAUTHORIZED {
                Self::AuthenticationError
            } else {
                Self::NetworkError(status_code, value.kind.to_string())
            }
        } else {
            match value.kind {
                tmdb_easy_raw::ErrorKind::NetworkError(error) => Self::RequestCreationError(error),
                tmdb_easy_raw::ErrorKind::DeserializationError(_) => unreachable!(),
            }
        }
    }
}
