use reqwest::StatusCode;

#[derive(Debug)]
pub enum Error {
    NetworkError,
    AuthenticationError,
    ParsingError,
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
