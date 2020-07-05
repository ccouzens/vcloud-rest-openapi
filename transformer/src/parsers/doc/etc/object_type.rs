use crate::parsers::doc::etc::annotation::Annotation;
use crate::parsers::doc::etc::field::Field;
use crate::parsers::doc::etc::group_ref::GroupRef;
use crate::parsers::doc::etc::r#type::TypeParseError;
use crate::parsers::doc::etc::XML_SCHEMA_NS;
use std::convert::TryFrom;

#[derive(Debug, PartialEq)]
pub(super) struct ObjectType {
    pub(super) annotation: Option<Annotation>,
    pub(super) name: String,
    pub(super) fields: Vec<Field>,
    pub(super) parents: Vec<String>,
}

impl TryFrom<(&xmltree::XMLNode, &str)> for ObjectType {
    type Error = TypeParseError;

    fn try_from((xml, schema_namespace): (&xmltree::XMLNode, &str)) -> Result<Self, Self::Error> {
        match xml {
            xmltree::XMLNode::Element(xmltree::Element {
                namespace: Some(namespace),
                name,
                attributes,
                children,
                ..
            }) if namespace == XML_SCHEMA_NS && (name == "complexType" || name == "group") => {
                let name = attributes
                    .get("name")
                    .map(|n| format!("{}:{}", schema_namespace, n))
                    .ok_or(TypeParseError::MissingName)?;
                let annotation = children
                    .iter()
                    .filter_map(|c| Annotation::try_from(c).ok())
                    .next();
                let mut fields = Vec::new();
                let mut parents = Vec::new();
                fields.extend(
                    children
                        .iter()
                        .flat_map(|xml| Field::try_from((xml, schema_namespace))),
                );
                for child in children {
                    match child {
                        xmltree::XMLNode::Element(xmltree::Element {
                            namespace: Some(namespace),
                            name,
                            children,
                            ..
                        }) if namespace == XML_SCHEMA_NS && name == "sequence" => {
                            fields.extend(
                                children
                                    .iter()
                                    .flat_map(|xml| Field::try_from((xml, schema_namespace))),
                            );
                            parents.extend(
                                children
                                    .iter()
                                    .flat_map(GroupRef::try_from)
                                    .map(|g| g.reference),
                            );
                        }
                        xmltree::XMLNode::Element(xmltree::Element {
                            namespace: Some(namespace),
                            name,
                            children,
                            ..
                        }) if namespace == XML_SCHEMA_NS && name == "complexContent" => {
                            for child in children {
                                match child {
                                    xmltree::XMLNode::Element(xmltree::Element {
                                        attributes,
                                        namespace: Some(namespace),
                                        name,
                                        children,
                                        ..
                                    }) if namespace == XML_SCHEMA_NS && name == "extension" => {
                                        parents.extend(attributes.get("base").cloned());
                                        fields.extend(children.iter().flat_map(|xml| {
                                            Field::try_from((xml, schema_namespace))
                                        }));
                                        parents.extend(
                                            children
                                                .iter()
                                                .flat_map(GroupRef::try_from)
                                                .map(|g| g.reference),
                                        );
                                        for child in children {
                                            match child {
                                                xmltree::XMLNode::Element(xmltree::Element {
                                                    namespace: Some(namespace),
                                                    name,
                                                    children,
                                                    ..
                                                }) if namespace == XML_SCHEMA_NS
                                                    && name == "sequence" =>
                                                {
                                                    fields.extend(children.iter().flat_map(
                                                        |xml| {
                                                            Field::try_from((xml, schema_namespace))
                                                        },
                                                    ));
                                                    parents.extend(
                                                        children
                                                            .iter()
                                                            .flat_map(GroupRef::try_from)
                                                            .map(|g| g.reference),
                                                    );
                                                }
                                                _ => {}
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                        _ => {}
                    }
                }
                Ok(ObjectType {
                    name,
                    annotation,
                    fields,
                    parents: parents
                        .into_iter()
                        .map(|p| {
                            if p.contains(':') {
                                p
                            } else {
                                format!("{}:{}", schema_namespace, p)
                            }
                        })
                        .collect(),
                })
            }
            _ => Err(TypeParseError::NotTypeNode),
        }
    }
}

impl From<&ObjectType> for openapiv3::Schema {
    fn from(c: &ObjectType) -> Self {
        let schema_kind =
            openapiv3::SchemaKind::Type(openapiv3::Type::Object(openapiv3::ObjectType {
                properties: c
                    .fields
                    .iter()
                    .map(|s| {
                        (
                            s.name.clone(),
                            openapiv3::ReferenceOr::boxed_item(openapiv3::Schema::from(s)),
                        )
                    })
                    .collect(),
                additional_properties: Some(openapiv3::AdditionalProperties::Any(false)),
                required: c
                    .fields
                    .iter()
                    .filter_map(|s| {
                        if s.annotation
                            .as_ref()
                            .and_then(|a| a.required)
                            .unwrap_or(false)
                        {
                            Some(s.name.clone())
                        } else {
                            None
                        }
                    })
                    .collect(),
                ..Default::default()
            }));
        let schema_data = openapiv3::SchemaData {
            deprecated: c.annotation.as_ref().map(|a| a.deprecated).unwrap_or(false),
            title: Some(c.name.clone()),
            description: c.annotation.as_ref().map(|a| &a.description).cloned(),
            ..Default::default()
        };
        match &c.parents.is_empty() {
            true => openapiv3::Schema {
                schema_data,
                schema_kind,
            },
            false => {
                let mut all_of = Vec::new();
                all_of.extend(c.parents.iter().map(|reference| {
                    openapiv3::ReferenceOr::Reference {
                        reference: format!("#/components/schemas/{}", reference),
                    }
                }));

                all_of.push(openapiv3::ReferenceOr::Item(openapiv3::Schema {
                    schema_kind,
                    schema_data: Default::default(),
                }));

                openapiv3::Schema {
                    schema_data,
                    schema_kind: openapiv3::SchemaKind::AllOf { all_of },
                }
            }
        }
    }
}
