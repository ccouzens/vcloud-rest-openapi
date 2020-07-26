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
pub struct QueryParameter {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, PartialEq)]
pub struct Operation {
    pub method: Method,
    pub path: String,
    pub description: String,
    pub tag: &'static str,
    pub request_content: Option<(String, String)>,
    pub response_contents: Vec<(String, String)>,
    pub api_version: String,
    pub basic_auth: bool,
    pub deprecated: bool,
    pub query_parameters: Vec<QueryParameter>,
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
            .find("Description:")
            .and_then(DefinitionListValue::text_to_markdown)
            .ok_or(Self::Error::CannotFindDescriptionError)?;
        let tag = if path.starts_with("/admin/extension") {
            "extension"
        } else if path.starts_with("/admin") {
            "admin"
        } else {
            "user"
        };
        let request_content_type = p
            .definition_list
            .find("Input parameters")
            .and_then(DefinitionListValue::as_sublist)
            .and_then(|l| l.find("Consume media type(s):"))
            .and_then(DefinitionListValue::as_text)
            .and_then(|t| t.split("+xml<br>").next())
            .map(str::to_string);

        let request_content_ref = request_content_type
            .as_ref()
            .and_then(|c| content_type_mapping.get(c))
            .cloned();

        let request_content = match (request_content_type, request_content_ref) {
            (Some(t), Some(r)) => Some((t, r)),
            _ => None,
        };

        let response_contents = match p
            .definition_list
            .find("Output parameters")
            .and_then(DefinitionListValue::as_sublist)
            .and_then(|l| l.find("Produce media type(s):"))
            .and_then(DefinitionListValue::as_text)
        {
            Some(t) => t
                .split("<br>")
                .map(|t| {
                    t.trim_end_matches("+xml")
                        .trim_end_matches("+json")
                        .to_string()
                })
                .filter_map(|t| content_type_mapping.get(&t).map(|c| (t, c.clone())))
                .collect(),
            None => Vec::new(),
        };

        let basic_auth = p
            .definition_list
            .find("Examples")
            .and_then(DefinitionListValue::as_sublist)
            .and_then(|d| d.find("Request"))
            .and_then(DefinitionListValue::as_text)
            .map(|t| t.contains("Authorization:&nbsp;Basic"))
            .unwrap_or(false);

        let deprecated = p.definition_list.filter("Deprecated:").any(|_| true);

        let query_parameters = match p
            .definition_list
            .find("Query parameters")
            .and_then(DefinitionListValue::as_sublist)
        {
            Some(s) => {
                s.0.iter()
                    .filter_map(|(key, value)| value.as_text().map(|v| (key, v)))
                    .fold((None, Vec::new()), |(name, mut acc), (key, value)| {
                        match (key.as_str(), name) {
                            ("Parameter", _) => (Some(value.to_string()), acc),
                            ("Documentation", Some(name)) => {
                                let description = html2md::parse_html(value).trim().to_string();
                                acc.push(QueryParameter {
                                    name,
                                    description: if description.is_empty() {
                                        None
                                    } else {
                                        Some(description)
                                    },
                                });
                                (None, acc)
                            }
                            (_, name) => (name, acc),
                        }
                    })
                    .1
            }
            None => Vec::new(),
        };

        Ok(Self {
            method,
            path,
            description,
            tag,
            request_content,
            response_contents,
            api_version,
            basic_auth,
            deprecated,
            query_parameters,
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
                        content: o
                            .response_contents
                            .iter()
                            .map(|(t, r)| {
                                (
                                    format!("{}+json;version={}", t, api_version),
                                    openapiv3::MediaType {
                                        schema: Some(openapiv3::ReferenceOr::Reference {
                                            reference: format!("#/components/schemas/{}", r),
                                        }),
                                        ..Default::default()
                                    },
                                )
                            })
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
            parameters: o
                .query_parameters
                .into_iter()
                .map(|qp| {
                    openapiv3::ReferenceOr::Item(openapiv3::Parameter::Query {
                        parameter_data: openapiv3::ParameterData {
                            description: qp.description,
                            required: false,
                            deprecated: None,
                            format: openapiv3::ParameterSchemaOrContent::Schema(
                                openapiv3::ReferenceOr::Reference {
                                    reference: format!(
                                        "#/components/schemas/query-parameter_{}",
                                        qp.name
                                    ),
                                },
                            ),
                            name: qp.name,
                            example: None,
                            examples: Default::default(),
                        },
                        allow_reserved: false,
                        style: openapiv3::QueryStyle::Form,
                        allow_empty_value: None,
                    })
                })
                .collect(),
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
            response_contents: vec![
                (
                    "application/vnd.vmware.admin.testo".into(),
                    "MyTypeO".into()
                ),
                (
                    "application/vnd.vmware.admin.testo".into(),
                    "MyTypeO".into()
                )
            ],
            api_version: "32.0".into(),
            basic_auth: false,
            deprecated: false,
            query_parameters: vec![
                QueryParameter {
                    name: "force".into(),
                    description: Some("Documentation for force".into())
                },
                QueryParameter {
                    name: "recursive".into(),
                    description: None
                }
            ]
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
          "parameters": [
            {
              "in": "query",
              "name": "force",
              "description": "Documentation for force",
              "schema": {
                "$ref": "#/components/schemas/query-parameter_force"
              },
              "style": "form"
            },
            {
              "in": "query",
              "name": "recursive",
              "schema": {
                "$ref": "#/components/schemas/query-parameter_recursive"
              },
              "style": "form"
            }
          ],
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
