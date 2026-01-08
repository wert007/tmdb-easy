use tmdb_easy_raw::types::{SearchTvResponse200Results, TvSeasonDetailsResponse200};

use crate::{MovieOrTvLike, client::TmdbClient, error::Error};

pub trait TvLike: MovieOrTvLike {
    fn season(
        &self,
        client: &TmdbClient,
        season: u32,
    ) -> Result<TvSeasonDetailsResponse200, Error> {
        Ok(
            tmdb_easy_raw::parametrized_functions::tv_season_details_with_parameter(
                &client.client,
                &client.api_key,
                self.id() as _,
                season as _,
                Default::default(),
            )?,
        )
    }
}

impl MovieOrTvLike for SearchTvResponse200Results {
    fn id(&self) -> u64 {
        self.id as _
    }

    fn poster_path(&self, _client: &TmdbClient) -> Result<String, Error> {
        Ok(self.poster_path.clone())
    }
}
