use std::borrow::Cow;

use tmdb_easy_raw::types::ConfigurationDetailsResponse200;

use crate::{error::Error, search::movie::SearchMovieBuilder};

pub struct TmdbClient {
    pub(crate) client: reqwest::blocking::Client,
    pub(crate) api_key: Cow<'static, str>,
    pub(crate) configuration: Option<tmdb_easy_raw::types::ConfigurationDetailsResponse200>,
}

impl TmdbClient {
    pub fn new(api_key: impl Into<Cow<'static, str>>) -> Self {
        Self {
            client: reqwest::blocking::Client::new(),
            api_key: api_key.into(),
            configuration: None,
        }
    }

    pub fn search_for_movie<'a, 'b: 'a>(
        &'a self,
        name: impl Into<Cow<'b, str>>,
    ) -> SearchMovieBuilder<'a> {
        SearchMovieBuilder::new(self, name.into())
    }

    pub fn configuration_details(&mut self) -> Result<&ConfigurationDetailsResponse200, Error> {
        if self.configuration.is_none() {
            self.configuration = Some(tmdb_easy_raw::functions::configuration_details(
                &self.client,
                &self.api_key,
            )?);
        }
        Ok(self.configuration.as_ref().unwrap())
    }

    pub(crate) fn resolve_image_path(
        &mut self,
        poster_path: String,
    ) -> Result<image::DynamicImage, Error> {
        let configuration = self.configuration_details()?;
        let bytes = reqwest::blocking::get(format!(
            "{}{}{}",
            configuration.images.base_url,
            configuration
                .images
                .poster_sizes
                .last()
                .expect("at least one size?"),
            poster_path
        ))
        .map_err(|_| Error::NetworkError)?
        .bytes()
        .map_err(|_| Error::NetworkError)?;
        let img = image::ImageReader::new(std::io::Cursor::new(bytes))
            .with_guessed_format()
            .map_err(|_| Error::DecodingError)?
            .decode()
            .map_err(|_| Error::DecodingError)?;
        Ok(img)
    }
}
