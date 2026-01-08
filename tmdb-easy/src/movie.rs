use crate::{MovieOrTvLike, client::TmdbClient, error::Error};

impl MovieOrTvLike for tmdb_easy_raw::types::SearchMovieResponse200Results {
    fn id(&self) -> u64 {
        self.id as _
    }

    fn poster_path(&self, _: &TmdbClient) -> Result<String, Error> {
        Ok(self.poster_path.clone())
    }
}

impl MovieOrTvLike for tmdb_easy_raw::types::MovieDetailsResponse200 {
    fn id(&self) -> u64 {
        self.id as _
    }

    fn poster_path(&self, _client: &TmdbClient) -> Result<String, Error> {
        Ok(self.poster_path.clone())
    }
}

#[test]
fn try_poster_download() -> Result<(), Error> {
    use crate::MovieOrTvLikeExt;
    let mut client = TmdbClient::new(include_str!("../../api_key.txt"));
    let movie = client.search_for_movie("Fall").search()?[0].clone();
    movie
        .poster(&mut client)?
        .save("fall-poster.png")
        .expect("works");
    Ok(())
}
