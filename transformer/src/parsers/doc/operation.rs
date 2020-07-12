use super::detail_page::{DefinitionListValue, DetailPage, DetailPageFromStrError};
#[cfg(test)]
use serde_json::json;
use std::{convert::TryFrom, str::FromStr};
use thiserror::Error;

#[derive(Debug, PartialEq)]
pub enum Method {
    Get,
    Post,
    Put,
    Delete,
}

#[derive(Error, Debug)]
pub enum MethodParseError {
    #[error("Unknown method `{0}`")]
    UnknownMethodError(String),
}

impl FromStr for Method {
    type Err = MethodParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "GET" => Ok(Self::Get),
            "POST" => Ok(Self::Post),
            "PUT" => Ok(Self::Put),
            "DELETE" => Ok(Self::Delete),
            other => Err(Self::Err::UnknownMethodError(other.into())),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Operation {
    pub method: Method,
    pub path: String,
    pub description: String,
    pub tag: &'static str,
}

#[derive(Error, Debug)]
pub enum OperationParseError {
    #[error("Detail Page Parse Error `{0}`")]
    DetailPageParseError(#[from] DetailPageFromStrError),
    #[error("Cannot find method")]
    CannotFindMethodError,
    #[error("Cannot parse method `{0}`")]
    MethodParseError(#[from] MethodParseError),
    #[error("Cannot find path")]
    CannotFindPathError,
    #[error("Cannot find description")]
    CannotFindDescriptionError,
    #[error("Description had unexpected type")]
    DescriptionWrongType,
}

impl TryFrom<&str> for Operation {
    type Error = OperationParseError;

    fn try_from(html: &str) -> Result<Self, Self::Error> {
        Ok(Self::try_from(DetailPage::try_from(html)?)?)
    }
}

impl TryFrom<DetailPage> for Operation {
    type Error = OperationParseError;

    fn try_from(p: DetailPage) -> Result<Self, Self::Error> {
        let method =
            p.h1.split_ascii_whitespace()
                .next()
                .ok_or(Self::Error::CannotFindMethodError)?
                .parse()?;
        let path: String =
            p.h1.splitn(2, ' ')
                .nth(1)
                .ok_or(Self::Error::CannotFindPathError)?
                .into();
        let description = p
            .definition_list
            .0
            .get("Description:")
            .and_then(DefinitionListValue::text_to_markdown)
            .ok_or(Self::Error::CannotFindDescriptionError)?;
        let tag = if path.starts_with("/admin/extension") {
            "extension"
        } else if path.starts_with("/admin") {
            "admin"
        } else {
            "user"
        };
        Ok(Self {
            method,
            path,
            description,
            tag,
        })
    }
}

impl From<Operation> for openapiv3::Operation {
    fn from(o: Operation) -> Self {
        openapiv3::Operation {
            description: Some(o.description),
            responses: openapiv3::Responses {
                responses: [(
                    openapiv3::StatusCode::Range(2),
                    openapiv3::ReferenceOr::Item(openapiv3::Response {
                        description: "success".into(),
                        ..Default::default()
                    }),
                )]
                .iter()
                .cloned()
                .collect(),
                ..Default::default()
            },
            tags: vec![o.tag.into()],
            ..Default::default()
        }
    }
}

#[test]
fn parse_operation_test() {
    let actual = Operation::try_from(include_str!("operations/PUT-Test.html")).unwrap();
    assert_eq!(
        actual,
        Operation {
            method: Method::Put,
            path: "/admin/test/{id}".into(),
            description: "Update a test.".into(),
            tag: "admin"
        }
    )
}

#[test]
fn generate_schema_test() {
    let op = Operation::try_from(include_str!("operations/PUT-Test.html")).unwrap();
    let value = openapiv3::Operation::from(op);
    assert_eq!(
        serde_json::to_value(value).unwrap(),
        json!({
            "tags": [ "admin" ],
            "description": "Update a test.",
            "responses": {
                "2XX": {
                    "description": "success"
                }
            }
        })
    )
}
