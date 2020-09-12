use indexmap::IndexMap;
use openapiv3::{ReferenceOr, Schema};
use std::collections::BTreeMap;
use std::convert::TryFrom;
use std::io::{Read, Seek};
use zip::read::ZipArchive;

pub fn schemas<R: Read + Seek>(
    zip: &mut ZipArchive<R>,
) -> Result<
    (
        IndexMap<String, ReferenceOr<Schema>>,
        BTreeMap<String, String>,
    ),
    Box<dyn std::error::Error>,
> {
    let mut output = query_parameters();
    let mut type_file_names = zip
        .file_names()
        .filter(|n| n.starts_with("doc/etc/"))
        .filter(|n| n.ends_with(".xsd"))
        .filter(|&n| n != "doc/etc/schemas/external/xml.xsd")
        .filter(|n| !n.starts_with("doc/etc/schemas/external/ovf1.1/"))
        .map(|n| n.into())
        .collect::<Vec<String>>();

    type_file_names.sort();

    let mut content_type_mapping = BTreeMap::new();

    for type_file_name in type_file_names {
        let mut bytes = Vec::new();
        zip.by_name(&type_file_name)?.read_to_end(&mut bytes)?;

        let namespace = if type_file_name.starts_with("doc/etc/1.5/schemas/extension/") {
            "vcloud-ext"
        } else if type_file_name.starts_with("doc/etc/1.5/schemas/") {
            "vcloud"
        } else if type_file_name.starts_with("doc/etc/schemas/versioning/") {
            "versioning"
        } else if type_file_name.starts_with("doc/etc/schemas/external/ovf1.1/") {
            "ovf"
        } else {
            "unknown"
        };

        let xsd_schema =
            crate::parsers::doc::etc::schema::Schema::try_from((&bytes as &[u8], namespace))?;
        output.extend(
            Vec::<Schema>::from(&xsd_schema)
                .into_iter()
                .filter_map(|s| {
                    s.schema_data
                        .title
                        .clone()
                        .map(|title| (title, ReferenceOr::Item(s)))
                }),
        );
        content_type_mapping.extend(xsd_schema.content_types_names());
    }

    output.insert(
        "ovf_Section_Type".to_owned(),
        ReferenceOr::Item(openapiv3::Schema {
            schema_data: Default::default(),
            schema_kind: openapiv3::SchemaKind::Type(openapiv3::Type::Object(
                openapiv3::ObjectType {
                    properties: [(
                        "info".to_string(),
                        openapiv3::ReferenceOr::boxed_item(openapiv3::Schema {
                            schema_data: Default::default(),
                            schema_kind: openapiv3::SchemaKind::Type(openapiv3::Type::Object(
                                openapiv3::ObjectType {
                                    properties: [(
                                        "value".to_string(),
                                        openapiv3::ReferenceOr::boxed_item(openapiv3::Schema {
                                            schema_data: Default::default(),
                                            schema_kind: openapiv3::SchemaKind::Type(
                                                openapiv3::Type::String(Default::default()),
                                            ),
                                        }),
                                    )]
                                    .iter()
                                    .cloned()
                                    .collect(),
                                    ..Default::default()
                                },
                            )),
                        }),
                    )]
                    .iter()
                    .cloned()
                    .collect(),
                    ..Default::default()
                },
            )),
        }),
    );

    let metadata_types: Vec<(String, String)> = output
        .keys()
        .filter_map(|k| {
            k.strip_prefix("vcloud_").and_then(|t| {
                if t.starts_with("Metadata") && t.ends_with("Value") && t != "MetadataTypedValue" {
                    Some((String::from(k), String::from(t)))
                } else {
                    None
                }
            })
        })
        .collect();

    output.insert(
        String::from("MetadataTypedValue"),
        ReferenceOr::Item(Schema {
            schema_kind: openapiv3::SchemaKind::OneOf {
                one_of: metadata_types
                    .iter()
                    .map(|qrt| openapiv3::ReferenceOr::Reference {
                        reference: qrt.0.clone(),
                    })
                    .collect(),
            },
            schema_data: openapiv3::SchemaData {
                discriminator: Some(openapiv3::Discriminator {
                    property_name: String::from("_type"),
                    mapping: metadata_types
                        .iter()
                        .map(|qrt| (qrt.1.clone(), format!("#/components/schemas/{}", qrt.0)))
                        .collect(),
                }),
                ..Default::default()
            },
        }),
    );

    for schema in output.values_mut() {
        if let openapiv3::ReferenceOr::Item(openapiv3::Schema {
            schema_kind: openapiv3::SchemaKind::AllOf { all_of },
            ..
        }) = schema
        {
            if let Some(properties) = all_of.iter_mut().find_map(|ref_schema| match ref_schema {
                ReferenceOr::Item(openapiv3::Schema {
                    schema_kind:
                        openapiv3::SchemaKind::Type(openapiv3::Type::Object(openapiv3::ObjectType {
                            properties,
                            ..
                        })),
                    ..
                }) => Some(properties),
                _ => None,
            }) {
                if let Some(openapiv3::ReferenceOr::Item(typed_value)) =
                    properties.get_mut("typedValue")
                {
                    typed_value.schema_kind = openapiv3::SchemaKind::AllOf {
                        all_of: vec![openapiv3::ReferenceOr::Reference {
                            reference: String::from("#/components/schemas/MetadataTypedValue"),
                        }],
                    }
                }
            };
        };
    }

    if let Some(openapiv3::ReferenceOr::Item(openapiv3::Schema {
        schema_kind:
            openapiv3::SchemaKind::Type(openapiv3::Type::Object(openapiv3::ObjectType {
                properties,
                required,
                ..
            })),
        ..
    })) = output.get_mut("vcloud_MetadataTypedValue")
    {
        properties.entry(String::from("_type")).or_insert_with(|| {
            openapiv3::ReferenceOr::boxed_item(openapiv3::Schema {
                schema_data: Default::default(),
                schema_kind: openapiv3::SchemaKind::Type(openapiv3::Type::String(
                    openapiv3::StringType {
                        enumeration: metadata_types.into_iter().map(|qrt| qrt.1).collect(),
                        ..Default::default()
                    },
                )),
            })
        });
        if required.iter().find(|&r| r == "_type") == None {
            required.push(String::from("_type"));
        }
    }

    let query_record_types: Vec<(String, String)> = output
        .keys()
        .filter_map(|k| {
            k.strip_prefix("vcloud_").and_then(|t| {
                if t.starts_with("QueryResult")
                    && t.ends_with("RecordType")
                    && t != "QueryResultRecordType"
                {
                    Some((String::from(k), String::from(t)))
                } else {
                    None
                }
            })
        })
        .collect();

    output.insert(
        String::from("QueryResultRecordType"),
        ReferenceOr::Item(Schema {
            schema_kind: openapiv3::SchemaKind::OneOf {
                one_of: query_record_types
                    .iter()
                    .map(|qrt| openapiv3::ReferenceOr::Reference {
                        reference: qrt.0.clone(),
                    })
                    .collect(),
            },
            schema_data: openapiv3::SchemaData {
                discriminator: Some(openapiv3::Discriminator {
                    property_name: String::from("_type"),
                    mapping: query_record_types
                        .iter()
                        .map(|qrt| (qrt.1.clone(), format!("#/components/schemas/{}", qrt.0)))
                        .collect(),
                }),
                ..Default::default()
            },
        }),
    );

    if let Some(openapiv3::ReferenceOr::Item(openapiv3::Schema {
        schema_kind:
            openapiv3::SchemaKind::Type(openapiv3::Type::Object(openapiv3::ObjectType {
                properties,
                ..
            })),
        ..
    })) = output.get_mut("vcloud_QueryResultRecordType")
    {
        properties.entry(String::from("_type")).or_insert_with(|| {
            openapiv3::ReferenceOr::boxed_item(openapiv3::Schema {
                schema_data: Default::default(),
                schema_kind: openapiv3::SchemaKind::Type(openapiv3::Type::String(
                    openapiv3::StringType {
                        enumeration: query_record_types.into_iter().map(|qrt| qrt.1).collect(),
                        ..Default::default()
                    },
                )),
            })
        });
    }

    if let Some(openapiv3::ReferenceOr::Item(openapiv3::Schema {
        schema_kind: openapiv3::SchemaKind::AllOf { all_of },
        ..
    })) = output.get_mut("vcloud_ContainerType")
    {
        if let Some(properties) = all_of.iter_mut().find_map(|ref_schema| match ref_schema {
            ReferenceOr::Item(openapiv3::Schema {
                schema_kind:
                    openapiv3::SchemaKind::Type(openapiv3::Type::Object(openapiv3::ObjectType {
                        properties,
                        ..
                    })),
                ..
            }) => Some(properties),
            _ => None,
        }) {
            properties.entry(String::from("record")).or_insert_with(|| {
                openapiv3::ReferenceOr::boxed_item(openapiv3::Schema {
                    schema_data: Default::default(),
                    schema_kind: openapiv3::SchemaKind::Type(openapiv3::Type::Array(
                        openapiv3::ArrayType {
                            items: openapiv3::ReferenceOr::Reference {
                                reference: String::from(
                                    "#/components/schemas/QueryResultRecordType",
                                ),
                            },
                            min_items: None,
                            max_items: None,
                            unique_items: false,
                        },
                    )),
                })
            });
            properties
                .entry(String::from("reference"))
                .or_insert_with(|| {
                    openapiv3::ReferenceOr::boxed_item(openapiv3::Schema {
                        schema_data: Default::default(),
                        schema_kind: openapiv3::SchemaKind::Type(openapiv3::Type::Array(
                            openapiv3::ArrayType {
                                items: openapiv3::ReferenceOr::Reference {
                                    reference: String::from(
                                        "#/components/schemas/vcloud_ReferenceType",
                                    ),
                                },
                                min_items: None,
                                max_items: None,
                                unique_items: false,
                            },
                        )),
                    })
                });
        };
    }

    Ok((output, content_type_mapping))
}

fn query_parameters() -> IndexMap<String, ReferenceOr<Schema>> {
    [
        (
            "force",
            ReferenceOr::Item(Schema {
                schema_data: Default::default(),
                schema_kind: openapiv3::SchemaKind::Type(openapiv3::Type::Boolean {}),
            }),
        ),
        (
            "recursive",
            ReferenceOr::Item(Schema {
                schema_data: Default::default(),
                schema_kind: openapiv3::SchemaKind::Type(openapiv3::Type::Boolean {}),
            }),
        ),
        (
            "fields",
            ReferenceOr::Item(Schema {
                schema_data: Default::default(),
                schema_kind: openapiv3::SchemaKind::Type(openapiv3::Type::String(
                    Default::default(),
                )),
            }),
        ),
        (
            "filter",
            ReferenceOr::Item(Schema {
                schema_data: Default::default(),
                schema_kind: openapiv3::SchemaKind::Type(openapiv3::Type::String(
                    Default::default(),
                )),
            }),
        ),
        (
            "filterEncoded",
            ReferenceOr::Item(Schema {
                schema_data: Default::default(),
                schema_kind: openapiv3::SchemaKind::Type(openapiv3::Type::Boolean {}),
            }),
        ),
        (
            "format",
            ReferenceOr::Item(Schema {
                schema_data: Default::default(),
                schema_kind: openapiv3::SchemaKind::Type(openapiv3::Type::String(
                    openapiv3::StringType {
                        enumeration: ["references", "records", "idrecords"]
                            .iter()
                            .map(|e| e.to_string())
                            .collect(),
                        ..Default::default()
                    },
                )),
            }),
        ),
        (
            "links",
            ReferenceOr::Item(Schema {
                schema_data: Default::default(),
                schema_kind: openapiv3::SchemaKind::Type(openapiv3::Type::Boolean {}),
            }),
        ),
        (
            "offset",
            ReferenceOr::Item(Schema {
                schema_data: Default::default(),
                schema_kind: openapiv3::SchemaKind::Type(openapiv3::Type::Integer(
                    openapiv3::IntegerType {
                        minimum: Some(0),
                        ..Default::default()
                    },
                )),
            }),
        ),
        (
            "page",
            ReferenceOr::Item(Schema {
                schema_data: Default::default(),
                schema_kind: openapiv3::SchemaKind::Type(openapiv3::Type::Integer(
                    openapiv3::IntegerType {
                        minimum: Some(1),
                        ..Default::default()
                    },
                )),
            }),
        ),
        (
            "pageSize",
            ReferenceOr::Item(Schema {
                schema_data: Default::default(),
                schema_kind: openapiv3::SchemaKind::Type(openapiv3::Type::Integer(
                    openapiv3::IntegerType {
                        minimum: Some(1),
                        maximum: Some(128),
                        ..Default::default()
                    },
                )),
            }),
        ),
        (
            "sortAsc",
            ReferenceOr::Item(Schema {
                schema_data: Default::default(),
                schema_kind: openapiv3::SchemaKind::Type(openapiv3::Type::String(
                    Default::default(),
                )),
            }),
        ),
        (
            "sortDesc",
            ReferenceOr::Item(Schema {
                schema_data: Default::default(),
                schema_kind: openapiv3::SchemaKind::Type(openapiv3::Type::String(
                    Default::default(),
                )),
            }),
        ),
        (
            "type",
            ReferenceOr::Item(Schema {
                schema_data: Default::default(),
                schema_kind: openapiv3::SchemaKind::Type(openapiv3::Type::String(
                    Default::default(),
                )),
            }),
        ),
    ]
    .iter()
    .map(|(name, schema)| (format!("query-parameter_{}", name), schema.clone()))
    .collect()
}
