use reqwest::StatusCode;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Network error")]
    NetworkError,
    #[error("Problem authenticating, check your api key")]
    AuthenticationError,
    #[error("Problem parsing answer from api, update dependency")]
    ParsingError,
    #[error("Problem reading image data")]
    DecodingError,
}

impl From<tmdb_easy_raw::Error> for Error {
    fn from(value: tmdb_easy_raw::Error) -> Self {
        if let Some(_text) = value.context.response_text {
            Self::ParsingError
        } else if let Some(status_code) = value.context.response_status {
            if status_code == StatusCode::UNAUTHORIZED {
                Self::AuthenticationError
            } else {
                Self::NetworkError
            }
        } else {
            Self::NetworkError
        }
    }
}
