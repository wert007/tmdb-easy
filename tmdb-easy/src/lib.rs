pub mod client;
pub mod error;
pub mod movie;
mod search;
pub mod tv;
use image::DynamicImage;
pub use tmdb_easy_raw;

use crate::{client::TmdbClient, error::Error};

pub trait MovieOrTvLike {
    fn id(&self) -> u64;
    fn poster_path(&self, client: &TmdbClient) -> Result<String, Error>;
}

pub trait MovieOrTvLikeExt: MovieOrTvLike {
    fn poster(&self, client: &mut TmdbClient) -> Result<DynamicImage, Error> {
        let poster_path = self.poster_path(client)?;
        client.resolve_image_path(poster_path)
    }
}

impl<T: MovieOrTvLike> MovieOrTvLikeExt for T {}
