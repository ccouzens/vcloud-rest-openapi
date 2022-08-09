use indexmap::IndexMap;
use openapiv3::{ObjectType, ReferenceOr, Schema, SchemaKind, Type};

pub fn stub_ovf(output: &mut IndexMap<String, ReferenceOr<Schema>>) {
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
    output.insert(
        "ovf_Item".to_owned(),
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
    output.insert(
        "ovf_RASD_Type".to_owned(),
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
}
