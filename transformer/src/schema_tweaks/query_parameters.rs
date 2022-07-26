use indexmap::IndexMap;
use openapiv3::{IntegerType, ReferenceOr, Schema, SchemaKind, StringType, Type};

use crate::parsers::doc::query::Query;

pub fn query_parameters(schemas: &mut IndexMap<String, ReferenceOr<Schema>>, queries: &[Query]) {
    schemas.extend(
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
                            .map(|e| Some(e.to_string()))
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
                    schema_kind: SchemaKind::Type(Type::String(StringType {
                        enumeration: queries.iter().map(|q| Some(q.name.clone())).collect(),
                        ..Default::default()
                    })),
                }),
            ),
        ]
        .iter()
        .map(|(name, schema)| (format!("query-parameter_{}", name), schema.clone())),
    )
}
