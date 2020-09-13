use indexmap::IndexMap;
use openapiv3::{
    ArrayType, Discriminator, IntegerType, ObjectType, ReferenceOr, Schema, SchemaData, SchemaKind,
    StringType, Type,
};
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
        ReferenceOr::Item(Schema {
            schema_data: Default::default(),
            schema_kind: SchemaKind::Type(Type::Object(ObjectType {
                properties: [(
                    "info".to_string(),
                    ReferenceOr::boxed_item(Schema {
                        schema_data: Default::default(),
                        schema_kind: SchemaKind::Type(Type::Object(ObjectType {
                            properties: [(
                                "value".to_string(),
                                ReferenceOr::boxed_item(Schema {
                                    schema_data: Default::default(),
                                    schema_kind: SchemaKind::Type(Type::String(Default::default())),
                                }),
                            )]
                            .iter()
                            .cloned()
                            .collect(),
                            ..Default::default()
                        })),
                    }),
                )]
                .iter()
                .cloned()
                .collect(),
                ..Default::default()
            })),
        }),
    );

    insert_metadata_superclass(&mut output);
    add_query_result_types(&mut output);

    Ok((output, content_type_mapping))
}

fn query_parameters() -> IndexMap<String, ReferenceOr<Schema>> {
    [
        (
            "force",
            ReferenceOr::Item(Schema {
                schema_data: Default::default(),
                schema_kind: SchemaKind::Type(Type::Boolean {}),
            }),
        ),
        (
            "recursive",
            ReferenceOr::Item(Schema {
                schema_data: Default::default(),
                schema_kind: SchemaKind::Type(Type::Boolean {}),
            }),
        ),
        (
            "fields",
            ReferenceOr::Item(Schema {
                schema_data: Default::default(),
                schema_kind: SchemaKind::Type(Type::String(Default::default())),
            }),
        ),
        (
            "filter",
            ReferenceOr::Item(Schema {
                schema_data: Default::default(),
                schema_kind: SchemaKind::Type(Type::String(Default::default())),
            }),
        ),
        (
            "filterEncoded",
            ReferenceOr::Item(Schema {
                schema_data: Default::default(),
                schema_kind: SchemaKind::Type(Type::Boolean {}),
            }),
        ),
        (
            "format",
            ReferenceOr::Item(Schema {
                schema_data: Default::default(),
                schema_kind: SchemaKind::Type(Type::String(StringType {
                    enumeration: ["references", "records", "idrecords"]
                        .iter()
                        .map(|e| e.to_string())
                        .collect(),
                    ..Default::default()
                })),
            }),
        ),
        (
            "links",
            ReferenceOr::Item(Schema {
                schema_data: Default::default(),
                schema_kind: SchemaKind::Type(Type::Boolean {}),
            }),
        ),
        (
            "offset",
            ReferenceOr::Item(Schema {
                schema_data: Default::default(),
                schema_kind: SchemaKind::Type(Type::Integer(IntegerType {
                    minimum: Some(0),
                    ..Default::default()
                })),
            }),
        ),
        (
            "page",
            ReferenceOr::Item(Schema {
                schema_data: Default::default(),
                schema_kind: SchemaKind::Type(Type::Integer(IntegerType {
                    minimum: Some(1),
                    ..Default::default()
                })),
            }),
        ),
        (
            "pageSize",
            ReferenceOr::Item(Schema {
                schema_data: Default::default(),
                schema_kind: SchemaKind::Type(Type::Integer(IntegerType {
                    minimum: Some(1),
                    maximum: Some(128),
                    ..Default::default()
                })),
            }),
        ),
        (
            "sortAsc",
            ReferenceOr::Item(Schema {
                schema_data: Default::default(),
                schema_kind: SchemaKind::Type(Type::String(Default::default())),
            }),
        ),
        (
            "sortDesc",
            ReferenceOr::Item(Schema {
                schema_data: Default::default(),
                schema_kind: SchemaKind::Type(Type::String(Default::default())),
            }),
        ),
        (
            "type",
            ReferenceOr::Item(Schema {
                schema_data: Default::default(),
                schema_kind: SchemaKind::Type(Type::String(Default::default())),
            }),
        ),
    ]
    .iter()
    .map(|(name, schema)| (format!("query-parameter_{}", name), schema.clone()))
    .collect()
}

fn insert_metadata_superclass(output: &mut IndexMap<String, ReferenceOr<Schema>>) {
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
            schema_kind: SchemaKind::OneOf {
                one_of: metadata_types
                    .iter()
                    .map(|qrt| ReferenceOr::Reference {
                        reference: qrt.0.clone(),
                    })
                    .collect(),
            },
            schema_data: SchemaData {
                discriminator: Some(Discriminator {
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
        if let ReferenceOr::Item(Schema {
            schema_kind: SchemaKind::AllOf { all_of },
            ..
        }) = schema
        {
            if let Some(properties) = all_of.iter_mut().find_map(|ref_schema| match ref_schema {
                ReferenceOr::Item(Schema {
                    schema_kind: SchemaKind::Type(Type::Object(ObjectType { properties, .. })),
                    ..
                }) => Some(properties),
                _ => None,
            }) {
                if let Some(ReferenceOr::Item(typed_value)) = properties.get_mut("typedValue") {
                    typed_value.schema_kind = SchemaKind::AllOf {
                        all_of: vec![ReferenceOr::Reference {
                            reference: String::from("#/components/schemas/MetadataTypedValue"),
                        }],
                    }
                }
            };
        };
    }

    if let Some(ReferenceOr::Item(Schema {
        schema_kind:
            SchemaKind::Type(Type::Object(ObjectType {
                properties,
                required,
                ..
            })),
        ..
    })) = output.get_mut("vcloud_MetadataTypedValue")
    {
        properties.entry(String::from("_type")).or_insert_with(|| {
            ReferenceOr::boxed_item(Schema {
                schema_data: Default::default(),
                schema_kind: SchemaKind::Type(Type::String(StringType {
                    enumeration: metadata_types.into_iter().map(|qrt| qrt.1).collect(),
                    ..Default::default()
                })),
            })
        });
        if required.iter().find(|&r| r == "_type") == None {
            required.push(String::from("_type"));
        }
    }
}

fn add_query_result_types(output: &mut IndexMap<String, ReferenceOr<Schema>>) {
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
            schema_kind: SchemaKind::OneOf {
                one_of: query_record_types
                    .iter()
                    .map(|qrt| ReferenceOr::Reference {
                        reference: qrt.0.clone(),
                    })
                    .collect(),
            },
            schema_data: SchemaData {
                discriminator: Some(Discriminator {
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

    if let Some(ReferenceOr::Item(Schema {
        schema_kind: SchemaKind::Type(Type::Object(ObjectType { properties, .. })),
        ..
    })) = output.get_mut("vcloud_QueryResultRecordType")
    {
        properties.entry(String::from("_type")).or_insert_with(|| {
            ReferenceOr::boxed_item(Schema {
                schema_data: Default::default(),
                schema_kind: SchemaKind::Type(Type::String(StringType {
                    enumeration: query_record_types.into_iter().map(|qrt| qrt.1).collect(),
                    ..Default::default()
                })),
            })
        });
    }

    if let Some(ReferenceOr::Item(Schema {
        schema_kind: SchemaKind::AllOf { all_of },
        ..
    })) = output.get_mut("vcloud_ContainerType")
    {
        if let Some(properties) = all_of.iter_mut().find_map(|ref_schema| match ref_schema {
            ReferenceOr::Item(Schema {
                schema_kind: SchemaKind::Type(Type::Object(ObjectType { properties, .. })),
                ..
            }) => Some(properties),
            _ => None,
        }) {
            properties.entry(String::from("record")).or_insert_with(|| {
                ReferenceOr::boxed_item(Schema {
                    schema_data: Default::default(),
                    schema_kind: SchemaKind::Type(Type::Array(ArrayType {
                        items: ReferenceOr::Reference {
                            reference: String::from("#/components/schemas/QueryResultRecordType"),
                        },
                        min_items: None,
                        max_items: None,
                        unique_items: false,
                    })),
                })
            });
            properties
                .entry(String::from("reference"))
                .or_insert_with(|| {
                    ReferenceOr::boxed_item(Schema {
                        schema_data: Default::default(),
                        schema_kind: SchemaKind::Type(Type::Array(ArrayType {
                            items: ReferenceOr::Reference {
                                reference: String::from(
                                    "#/components/schemas/vcloud_ReferenceType",
                                ),
                            },
                            min_items: None,
                            max_items: None,
                            unique_items: false,
                        })),
                    })
                });
        };
    }
}
