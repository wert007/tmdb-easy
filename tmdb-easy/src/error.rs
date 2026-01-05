#[derive(Debug)]
pub enum Error {
    NetworkError,
    AuthenticationError,
    ParsingError,
    DecodingError,
}

impl From<tmdb_easy_raw::Error> for Error {
    fn from(value: tmdb_easy_raw::Error) -> Self {
        todo!()
    }
}
