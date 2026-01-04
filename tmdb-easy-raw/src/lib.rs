#![cfg_attr(docsrs, feature(doc_auto_cfg))]

//! This is a very raw version of the tmdb api. It is automatically built from
//! the open-api-schema. There are both async and blocking endpoints available.
//!
//! Right now it is heavily bundled with the reqwest crate.
//!
//! # Features
//! - **vendored** _(default-feature)_: Uses a predownloaded version of the open
//!   api schema, if disabled, it will download the schema during build time.
//! - **async**: Uses reqwest::Client to make requests to endpoints. Allows
//!   access to async functions.
//! - **blocking**: Uses reqwest::blocking::Client to make requests to
//!   endpoints. Allows access to blocking functions.

/// All the types that are returned or are used as parameters. Each type has a
/// Default::default implementation and can be parsed from json. There might be
/// a lot of duplication.
pub mod types {
    include!(concat!(env!("OUT_DIR"), "/types.rs"));
}

/// Blocking functions to access the whole tmdb api. Needs the "blocking"
/// feature to be active.
#[cfg(feature = "blocking")]
pub mod functions {
    include!(concat!(env!("OUT_DIR"), "/functions.rs"));
}

/// Async functions to access the whole tmdb api. Needs the "async" feature to
/// be active.
#[cfg(feature = "async")]
pub mod async_functions {
    include!(concat!(env!("OUT_DIR"), "/async_functions.rs"));
}

/// These are the parameter types, that contain all the optional parameters to
/// each endpoint. This allows for more ergonomic access to each endpoint,
/// without having to repeat something like `None, None, Some(1), None`,
/// everywhere.
pub mod parameter_types {
    include!(concat!(env!("OUT_DIR"), "/parameter_types.rs"));
}

/// Blocking functions, that use the types from crate::parameter_types to access
/// the whole tmdb api. Needs the "blocking" feature to be active.
#[cfg(feature = "blocking")]
pub mod parametrized_functions {
    include!(concat!(env!("OUT_DIR"), "/parametrized_functions.rs"));
}

/// Async functions, that use the types from crate::parameter_types to access
/// the whole tmdb api. Needs the "async" feature to be active.
#[cfg(feature = "async")]
pub mod async_parametrized_functions {
    include!(concat!(env!("OUT_DIR"), "/async_parametrized_functions.rs"));
}
