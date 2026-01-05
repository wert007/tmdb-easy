use std::{borrow::Cow, ops::Index};

use crate::client::TmdbClient;

pub struct SearchMovieResponse<'a> {
    builder: SearchMovieBuilder<'a>,
    results: tmdb_easy_raw::types::SearchMovieResponse200,
}

impl<'a> Index<usize> for SearchMovieResponse<'a> {
    type Output = tmdb_easy_raw::types::SearchMovieResponse200Results;

    fn index(&self, index: usize) -> &Self::Output {
        &self.results.results[index]
    }
}

impl<'a> SearchMovieResponse<'a> {
    // pub fn all_results(self) -> Result<Vec
    pub fn next_page(self) -> Option<Result<Self, tmdb_easy_raw::Error>> {
        if self.results.page >= self.results.total_pages {
            None
        } else {
            Some(self.builder.with_page(self.results.page + 1).search())
        }
    }
}

pub struct SearchMovieBuilder<'a> {
    client: &'a TmdbClient,
    query: Cow<'a, str>,
    parameters: tmdb_easy_raw::parameter_types::SearchMovieParameter<'a>,
}

impl<'a> SearchMovieBuilder<'a> {
    pub fn new(client: &'a TmdbClient, query: impl Into<Cow<'a, str>>) -> Self {
        Self {
            client,
            query: query.into(),
            parameters: Default::default(),
        }
    }

    pub fn with_year(mut self, year: u16) -> Self {
        self.parameters.year = Some(year.to_string().into());
        self
    }

    pub fn with_language<'b: 'a>(mut self, language: &'b str) -> Self {
        self.parameters.language = Some(language.into());
        self
    }

    pub fn with_page(mut self, page: i64) -> Self {
        self.parameters.page = Some(page);
        self
    }

    pub fn search(self) -> Result<SearchMovieResponse<'a>, tmdb_easy_raw::Error> {
        tmdb_easy_raw::parametrized_functions::search_movie_with_parameter(
            &self.client.client,
            self.client.api_key.as_ref(),
            &self.query,
            self.parameters.clone(),
        )
        .map(|results| SearchMovieResponse {
            builder: self,
            results,
        })
    }
}
