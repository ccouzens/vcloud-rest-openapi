use indexmap::IndexMap;
use openapiv3::{
    ArrayType, Discriminator, ObjectType, ReferenceOr, Schema, SchemaData, SchemaKind, StringType,
    Type,
};

pub fn query_superclass(output: &mut IndexMap<String, ReferenceOr<Schema>>) {
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
                        reference: format!("#/components/schemas/{}", qrt.0),
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
