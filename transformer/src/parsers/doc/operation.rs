use super::detail_page::{DefinitionListValue, DetailPage, DetailPageFromStrError};
use indexmap::map::IndexMap;
use indexmap::set::IndexSet;
#[cfg(test)]
use serde_json::json;
use std::collections::BTreeMap;
use std::{convert::TryFrom, str::FromStr};
use thiserror::Error;

#[derive(Debug, PartialEq, Copy, Clone)]
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
    pub request_contents: IndexSet<String>,
    pub response_description: Option<String>,
    pub response_contents: IndexSet<String>,
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
}

impl TryFrom<&str> for Operation {
    type Error = OperationParseError;

    fn try_from(html: &str) -> Result<Self, Self::Error> {
        Ok(Self::try_from(DetailPage::try_from(html)?)?)
    }
}

fn html_to_mimes<'a>(html: &'a str) -> impl Iterator<Item = String> + 'a {
    html.split("<br>")
        .filter(|&t| !(t.is_empty() || t == "None"))
        .map(String::from)
}

impl<'a> TryFrom<DetailPage> for Operation {
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
        let request_contents = p
            .definition_list
            .find("Input parameters")
            .and_then(DefinitionListValue::as_sublist)
            .and_then(|l| l.find("Consume media type(s):"))
            .and_then(DefinitionListValue::as_text)
            .map(|t| html_to_mimes(t).collect())
            .unwrap_or_default();

        let response_description = p
            .definition_list
            .find("Output parameters")
            .and_then(DefinitionListValue::as_text)
            .map(html2md::parse_html)
            .map(|s| s.trim().to_string());

        let response_contents = p
            .definition_list
            .find("Output parameters")
            .and_then(DefinitionListValue::as_sublist)
            .and_then(|l| l.find("Produce media type(s):"))
            .and_then(DefinitionListValue::as_text)
            .map(|t| html_to_mimes(t).collect())
            .unwrap_or_default();

        let basic_auth = p
            .definition_list
            .find("Examples")
            .and_then(DefinitionListValue::as_sublist)
            .and_then(|d| d.find("Request"))
            .and_then(DefinitionListValue::as_text)
            .map(|t| t.contains("Authorization:&nbsp;Basic"))
            .unwrap_or(false);

        let deprecated = p.definition_list.filter("Deprecated:").any(|_| true);

        let query_parameters = p
            .definition_list
            .find("Query parameters")
            .and_then(DefinitionListValue::as_sublist)
            .map(|s| {
                s.0.iter()
                    .filter_map(|(key, value)| value.as_text().map(|v| (key, v)))
                    .fold((None, Vec::new()), |(name, mut acc), (key, value)| {
                        match (key.as_str(), name) {
                            ("Parameter", _) => (Some(value.to_string()), acc),
                            ("Documentation", Some(name)) => {
                                let description = if value.is_empty() {
                                    None
                                } else {
                                    Some(html2md::parse_html(value))
                                };
                                acc.push(QueryParameter { name, description });
                                (None, acc)
                            }
                            (_, name) => (name, acc),
                        }
                    })
                    .1
            })
            .unwrap_or_else(Vec::new);

        Ok(Self {
            method,
            path,
            description,
            tag,
            request_contents,
            response_description,
            response_contents,
            basic_auth,
            deprecated,
            query_parameters,
        })
    }
}

fn mimes_to_content(
    mimes: &IndexSet<String>,
    api_version: &str,
    type_mapping: &BTreeMap<String, String>,
) -> IndexMap<String, openapiv3::MediaType> {
    mimes
        .iter()
        .filter_map(|mime| {
            let mime_without_format = mime.trim_end_matches("+json").trim_end_matches("+xml");
            type_mapping.get(mime_without_format).map(|type_name| {
                (
                    format!("{}+json;version={}", mime_without_format, api_version),
                    openapiv3::MediaType {
                        schema: Some(openapiv3::ReferenceOr::Reference {
                            reference: format!("#/components/schemas/{}", type_name),
                        }),
                        ..Default::default()
                    },
                )
            })
        })
        .collect()
}

impl Operation {
    pub fn to_openapi(
        self,
        api_version: &str,
        type_mapping: &BTreeMap<String, String>,
    ) -> openapiv3::Operation {
        let request_content = mimes_to_content(&self.request_contents, api_version, type_mapping);

        openapiv3::Operation {
            description: Some(self.description),
            responses: openapiv3::Responses {
                responses: [(
                    openapiv3::StatusCode::Range(2),
                    openapiv3::ReferenceOr::Item(openapiv3::Response {
                        description: self
                            .response_description
                            .unwrap_or_else(|| "success".into()),
                        content: mimes_to_content(
                            &self.response_contents,
                            api_version,
                            type_mapping,
                        ),
                        ..Default::default()
                    }),
                )]
                .iter()
                .cloned()
                .collect(),
                ..Default::default()
            },
            tags: vec![self.tag.into()],
            request_body: if !request_content.is_empty() {
                Some(openapiv3::ReferenceOr::Item(openapiv3::RequestBody {
                    content: request_content,
                    required: true,
                    ..Default::default()
                }))
            } else {
                None
            },
            security: vec![indexmap! {
                if self.basic_auth { "basicAuth" } else { "bearerAuth" }.to_string() => vec![]
            }],
            deprecated: self.deprecated,
            parameters: self
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
    let actual = Operation::try_from(include_str!("operations/PUT-Test.html")).unwrap();
    assert_eq!(
        actual,
        Operation {
            method: Method::Put,
            path: "/admin/test/{id}".into(),
            description: "Update a test.".into(),
            tag: "admin",
            request_contents: indexset![
                "application/vnd.vmware.admin.test+xml".into(),
                "application/vnd.vmware.admin.test+json".into()
            ],
            response_description: Some("AdminTestType  \n\nExtended description".into()),
            response_contents: indexset![
                "application/vnd.vmware.admin.testo+xml".into(),
                "application/vnd.vmware.admin.testo+json".into()
            ],
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
    let op = Operation::try_from(include_str!("operations/PUT-Test.html")).unwrap();
    let value = op.to_openapi(
        "32.0",
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
    );
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
              "description": "AdminTestType  \n\nExtended description",
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
    let op = Operation::try_from(include_str!("operations/POST-Login.html")).unwrap();
    let value = op.to_openapi("32.0", &BTreeMap::new());

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
