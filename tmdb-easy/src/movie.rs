use image::DynamicImage;

use crate::{client::TmdbClient, error::Error};

pub trait MovieLike {
    fn id(&self) -> u64;
    fn poster_path(&self, client: &TmdbClient) -> Result<String, Error> {
        client.movie_details(self.id()).map(|m| m.poster_path)
    }
}

pub trait MovieLikeExt: MovieLike {
    fn poster(&self, client: &mut TmdbClient) -> Result<DynamicImage, Error> {
        let poster_path = self.poster_path(client)?;
        client.resolve_image_path(poster_path)
    }
}

impl<T: MovieLike> MovieLikeExt for T {}

impl MovieLike for tmdb_easy_raw::types::SearchMovieResponse200Results {
    fn id(&self) -> u64 {
        self.id as _
    }

    fn poster_path(&self, _: &TmdbClient) -> Result<String, Error> {
        Ok(self.poster_path.clone())
    }
}

impl MovieLike for tmdb_easy_raw::types::MovieDetailsResponse200 {
    fn id(&self) -> u64 {
        self.id as _
    }

    fn poster_path(&self, _client: &TmdbClient) -> Result<String, Error> {
        Ok(self.poster_path.clone())
    }
}

#[test]
fn try_poster_download() -> Result<(), Error> {
    let mut client = TmdbClient::new(include_str!("../../api_key.txt"));
    let movie = client.search_for_movie("Fall").search()?[0].clone();
    movie
        .poster(&mut client)?
        .save("fall-poster.png")
        .expect("works");
    Ok(())
}
