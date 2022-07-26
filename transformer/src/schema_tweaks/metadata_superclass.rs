use indexmap::IndexMap;
use openapiv3::{
    Discriminator, ObjectType, ReferenceOr, Schema, SchemaData, SchemaKind, StringType, Type,
};

pub fn metadata_superclass(output: &mut IndexMap<String, ReferenceOr<Schema>>) {
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
                        reference: format!("#/components/schemas/{}", qrt.0),
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
                    extensions: Default::default(),
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
                    enumeration: metadata_types.into_iter().map(|qrt| Some(qrt.1)).collect(),
                    ..Default::default()
                })),
            })
        });
        if required.iter().find(|&r| r == "_type") == None {
            required.push(String::from("_type"));
        }
    }
}
