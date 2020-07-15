use super::detail_page::{DefinitionListValue, DetailPage, DetailPageFromStrError};
#[cfg(test)]
use serde_json::json;
use std::collections::BTreeMap;
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
    pub request_content: Option<(String, String)>,
    pub response_content: Option<(String, String)>,
    pub api_version: String,
    pub basic_auth: bool,
    pub deprecated: bool,
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

impl TryFrom<(&str, &BTreeMap<String, String>, String)> for Operation {
    type Error = OperationParseError;

    fn try_from(
        (html, content_type_mapping, api_version): (&str, &BTreeMap<String, String>, String),
    ) -> Result<Self, Self::Error> {
        Ok(Self::try_from((
            DetailPage::try_from(html)?,
            content_type_mapping,
            api_version,
        ))?)
    }
}

impl<'a> TryFrom<(DetailPage, &BTreeMap<String, String>, String)> for Operation {
    type Error = OperationParseError;

    fn try_from(
        (p, content_type_mapping, api_version): (DetailPage, &BTreeMap<String, String>, String),
    ) -> Result<Self, Self::Error> {
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
        let request_content_type = match p.definition_list.0.get("Input parameters") {
            Some(DefinitionListValue::SubList(b)) => match b.0.get("Consume media type(s):") {
                Some(DefinitionListValue::Text(t)) => t.split("+xml<br>").next(),
                _ => None,
            },
            _ => None,
        }
        .map(str::to_string);

        let request_content_ref = request_content_type
            .as_ref()
            .and_then(|c| content_type_mapping.get(c))
            .cloned();

        let request_content = match (request_content_type, request_content_ref) {
            (Some(t), Some(r)) => Some((t, r)),
            _ => None,
        };

        let response_content_type = match p.definition_list.0.get("Output parameters") {
            Some(DefinitionListValue::SubList(b)) => match b.0.get("Produce media type(s):") {
                Some(DefinitionListValue::Text(t)) => t.split("+xml<br>").next(),
                _ => None,
            },
            _ => None,
        }
        .map(str::to_string);

        let response_content_ref = response_content_type
            .as_ref()
            .and_then(|c| content_type_mapping.get(c))
            .cloned();

        let response_content = match (response_content_type, response_content_ref) {
            (Some(t), Some(r)) => Some((t, r)),
            _ => None,
        };

        let basic_auth = p
            .definition_list
            .0
            .get("Examples")
            .and_then(DefinitionListValue::as_sublist)
            .and_then(|d| d.0.get("Request"))
            .and_then(DefinitionListValue::as_text)
            .map(|t| t.contains("Authorization:&nbsp;Basic"))
            .unwrap_or(false);

        let deprecated = p.definition_list.0.contains_key("Deprecated:");

        Ok(Self {
            method,
            path,
            description,
            tag,
            request_content,
            response_content,
            api_version,
            basic_auth,
            deprecated,
        })
    }
}

impl From<Operation> for openapiv3::Operation {
    fn from(o: Operation) -> Self {
        let api_version = o.api_version;
        openapiv3::Operation {
            description: Some(o.description),
            responses: openapiv3::Responses {
                responses: [(
                    openapiv3::StatusCode::Range(2),
                    openapiv3::ReferenceOr::Item(openapiv3::Response {
                        description: "success".into(),
                        content: [o.response_content.map(|(t, r)| {
                            (
                                format!("{}+json;version={}", t, api_version),
                                openapiv3::MediaType {
                                    schema: Some(openapiv3::ReferenceOr::Reference {
                                        reference: format!("#/components/schemas/{}", r),
                                    }),
                                    ..Default::default()
                                },
                            )
                        })]
                        .iter()
                        .cloned()
                        .flatten()
                        .collect(),
                        ..Default::default()
                    }),
                )]
                .iter()
                .cloned()
                .collect(),
                ..Default::default()
            },
            tags: vec![o.tag.into()],
            request_body: o.request_content.map(|(t, r)| {
                openapiv3::ReferenceOr::Item(openapiv3::RequestBody {
                    content: [(
                        format!("{}+json;version={}", t, api_version),
                        openapiv3::MediaType {
                            schema: Some(openapiv3::ReferenceOr::Reference {
                                reference: format!("#/components/schemas/{}", r),
                            }),
                            ..Default::default()
                        },
                    )]
                    .iter()
                    .cloned()
                    .collect(),
                    required: true,
                    ..Default::default()
                })
            }),
            security: vec![indexmap! {
                if o.basic_auth { "basicAuth" } else { "bearerAuth" }.to_string() => vec![]
            }],
            deprecated: o.deprecated,
            ..Default::default()
        }
    }
}

#[test]
fn parse_operation_test() {
    let actual = Operation::try_from((
        include_str!("operations/PUT-Test.html"),
        &[
            (
                "application/vnd.vmware.admin.test".to_string(),
                "MyType".to_string(),
            ),
            (
                "application/vnd.vmware.admin.testo".to_string(),
                "MyTypeO".to_string(),
            ),
        ]
        .iter()
        .cloned()
        .collect(),
        "32.0".into(),
    ))
    .unwrap();
    assert_eq!(
        actual,
        Operation {
            method: Method::Put,
            path: "/admin/test/{id}".into(),
            description: "Update a test.".into(),
            tag: "admin",
            request_content: Some(("application/vnd.vmware.admin.test".into(), "MyType".into())),
            response_content: Some((
                "application/vnd.vmware.admin.testo".into(),
                "MyTypeO".into()
            )),
            api_version: "32.0".into(),
            basic_auth: false,
            deprecated: false
        }
    )
}

#[test]
fn generate_schema_test() {
    let op = Operation::try_from((
        include_str!("operations/PUT-Test.html"),
        &[
            (
                "application/vnd.vmware.admin.test".to_string(),
                "MyType".to_string(),
            ),
            (
                "application/vnd.vmware.admin.testo".to_string(),
                "MyTypeO".to_string(),
            ),
        ]
        .iter()
        .cloned()
        .collect(),
        "32.0".into(),
    ))
    .unwrap();
    let value = openapiv3::Operation::from(op);
    assert_eq!(
        serde_json::to_value(value).unwrap(),
        json!({
          "tags": [
            "admin"
          ],
          "description": "Update a test.",
          "requestBody": {
            "content": {
              "application/vnd.vmware.admin.test+json;version=32.0": {
                "schema": {
                  "$ref": "#/components/schemas/MyType"
                }
              }
            },
            "required": true
          },
          "responses": {
            "2XX": {
              "description": "success",
              "content": {
                "application/vnd.vmware.admin.testo+json;version=32.0": {
                  "schema": {
                    "$ref": "#/components/schemas/MyTypeO"
                  }
                }
              }
            }
          },
          "security": [
            {
              "bearerAuth": []
            }
          ]
        })
    )
}

#[test]
fn generate_schema_test_for_basic_auth() {
    let op = Operation::try_from((
        include_str!("operations/POST-Login.html"),
        &BTreeMap::new(),
        "32.0".into(),
    ))
    .unwrap();
    let value = openapiv3::Operation::from(op);

    assert_eq!(
        serde_json::to_value(value).unwrap(),
        json!({
          "tags": [
            "user"
          ],
          "description": "Log in.",
          "responses": {
            "2XX": {
              "description": "success"
            }
          },
          "deprecated": true,
          "security": [
            {
              "basicAuth": []
            }
          ]
        })
    )
}
