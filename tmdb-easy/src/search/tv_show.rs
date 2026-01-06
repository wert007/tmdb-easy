use std::{borrow::Cow, ops::Index};

use tmdb_easy_raw::types::SearchTvResponse200Results;

use crate::{client::TmdbClient, error::Error};

pub struct SearchTvResponse<'a> {
    builder: SearchTvBuilder<'a>,
    results: tmdb_easy_raw::types::SearchTvResponse200,
}

impl<'a> Index<usize> for SearchTvResponse<'a> {
    type Output = tmdb_easy_raw::types::SearchTvResponse200Results;

    fn index(&self, index: usize) -> &Self::Output {
        &self.results.results[index]
    }
}

impl<'a> SearchTvResponse<'a> {
    pub fn current_page(&self) -> &[SearchTvResponse200Results] {
        &self.results.results
    }
    // pub fn all_results(self) -> Result<Vec
    pub fn next_page(self) -> Option<Result<Self, Error>> {
        if self.results.page >= self.results.total_pages {
            None
        } else {
            Some(self.builder.with_page(self.results.page + 1).search())
        }
    }
}

pub struct SearchTvBuilder<'a> {
    client: &'a TmdbClient,
    query: Cow<'a, str>,
    parameters: tmdb_easy_raw::parameter_types::SearchTvParameter<'a>,
}

impl<'a> SearchTvBuilder<'a> {
    pub fn new(client: &'a TmdbClient, query: impl Into<Cow<'a, str>>) -> Self {
        Self {
            client,
            query: query.into(),
            parameters: Default::default(),
        }
    }

    pub fn with_year(mut self, year: u16) -> Self {
        self.parameters.year = Some(year.into());
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

    pub fn search(self) -> Result<SearchTvResponse<'a>, Error> {
        match tmdb_easy_raw::parametrized_functions::search_tv_with_parameter(
            &self.client.client,
            self.client.api_key.as_ref(),
            &self.query,
            self.parameters.clone(),
        ) {
            Ok(it) => Ok(SearchTvResponse {
                builder: self,
                results: it,
            }),
            Err(e) => Err(e.into()),
        }
    }
}
