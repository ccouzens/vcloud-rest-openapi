use crate::parsers::doc::operation::{Method, Operation};
use anyhow::{Context, Result};
use openapiv3::Paths;
use std::collections::BTreeMap;
use std::{
    convert::TryFrom,
    io::{Read, Seek},
};
use zip::read::ZipArchive;

pub fn paths<R: Read + Seek>(
    zip: &mut ZipArchive<R>,
    content_type_mapping: BTreeMap<String, String>,
    content_element_mapping: BTreeMap<String, String>,
    api_version: String,
) -> Result<Paths> {
    let path_param_regex =
        regex::Regex::new(r"\{([^}]+)}").context("Unable to create path param regex")?;
    let mut path_file_names = zip
        .file_names()
        .filter(|n| n.starts_with("doc/operations/"))
        .filter(|n| n.ends_with(".html"))
        .map(|n| n.into())
        .collect::<Vec<String>>();

    path_file_names.sort();

    let mut paths = Paths::new();

    for file_name in path_file_names {
        let mut html = String::new();
        zip.by_name(&file_name)
            .with_context(|| format!("Unable to get file {}", file_name))?
            .read_to_string(&mut html)
            .with_context(|| format!("Unable to read file {}", file_name))?;

        let operation = Operation::try_from(html.as_str())
            .with_context(|| format!("Unable to convert file to operation {}", file_name))?;
        if !operation.path.starts_with('/') {
            continue;
        }
        if let openapiv3::ReferenceOr::Item(path_item) =
            paths.entry(operation.path.clone()).or_insert_with(|| {
                openapiv3::ReferenceOr::Item(openapiv3::PathItem {
                    parameters: path_param_regex
                        .captures_iter(&operation.path)
                        .map(|c| {
                            openapiv3::ReferenceOr::Item(openapiv3::Parameter::Path {
                                parameter_data: openapiv3::ParameterData {
                                    name: c[1].into(),
                                    required: true,
                                    description: None,
                                    deprecated: None,
                                    format: openapiv3::ParameterSchemaOrContent::Schema(
                                        openapiv3::ReferenceOr::Item(openapiv3::Schema {
                                            schema_data: Default::default(),
                                            schema_kind: openapiv3::SchemaKind::Type(
                                                openapiv3::Type::String(
                                                    openapiv3::StringType::default(),
                                                ),
                                            ),
                                        }),
                                    ),
                                    example: None,
                                    examples: Default::default(),
                                },
                                style: Default::default(),
                            })
                        })
                        .collect(),
                    ..Default::default()
                })
            })
        {
            let method = operation.method;
            let openapi_op = Some(operation.to_openapi(
                &api_version,
                &content_type_mapping,
                &content_element_mapping,
            ));
            match method {
                Method::Get => path_item.get = openapi_op,
                Method::Post => path_item.post = openapi_op,
                Method::Put => path_item.put = openapi_op,
                Method::Delete => path_item.delete = openapi_op,
            }
        };
    }
    Ok(paths)
}
