use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema {
    pub info: SchemaInfo,
    pub servers: SchemaServers,
    pub paths: SchemaPaths,
}
impl Schema {
    pub(crate) fn download() -> Schema {
        if cfg!(feature = "vendored") {
            serde_json::from_str(include_str!("tmdb-api.json")).unwrap()
        } else {
            reqwest::blocking::get("https://developer.themoviedb.org/openapi/tmdb-api.json")
                .unwrap()
                .json()
                .unwrap()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaInfo {
    pub title: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaServers(pub Vec<SchemaServerUrl>);
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaServerUrl {
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaPaths(HashMap<String, SchemaPath>);

impl IntoIterator for SchemaPaths {
    type Item = <HashMap<String, SchemaPath> as IntoIterator>::Item;

    type IntoIter = <HashMap<String, SchemaPath> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a SchemaPaths {
    type Item = <&'a HashMap<String, SchemaPath> as IntoIterator>::Item;

    type IntoIter = <&'a HashMap<String, SchemaPath> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        (&self.0).into_iter()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaPath {
    pub get: Option<PathRoute>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PathRoute {
    pub operation_id: String,
    pub summary: String,
    pub description: String,
    #[serde(default)]
    pub parameters: Vec<PathRouteParameter>,
    pub responses: HashMap<String, PathRouteResponse>,
    pub deprecated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathRouteResponse {
    pub description: String,
    pub content: HashMap<String, ResponseSchema>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseSchema {
    pub schema: TypeSchema,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathRouteParameter {
    pub name: String,
    pub r#in: ParameterLocation,
    pub schema: TypeSchema,
    pub description: Option<String>,
    #[serde(default)]
    pub required: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ParameterLocation {
    Path,
    Query,
}

impl ParameterLocation {
    pub fn path(self) -> bool {
        matches!(self, Self::Path)
    }

    pub fn query(self) -> bool {
        matches!(self, Self::Query)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum TypeSchema {
    Tagged(TaggedTypeSchema),
    Empty(HashMap<String, String>),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum TaggedTypeSchema {
    Object {
        properties: HashMap<String, TypeSchema>,
    },
    Boolean {
        default: Option<bool>,
    },
    String {
        default: Option<String>,
    },
    Integer {
        default: Option<i64>,
    },
    Number {
        default: Option<f64>,
    },
    Array {
        items: Box<TypeSchema>,
    },
    #[serde(untagged)]
    Empty,
}
