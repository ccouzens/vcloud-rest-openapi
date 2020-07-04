use crate::parsers::doc::etc::annotation::Annotation;
use crate::parsers::doc::etc::field::Field;
use crate::parsers::doc::etc::r#type::TypeParseError;
use crate::parsers::doc::etc::XML_SCHEMA_NS;
use std::convert::TryFrom;

#[derive(Debug, PartialEq)]
pub(super) struct ObjectType {
    pub(super) annotation: Annotation,
    pub(super) name: String,
    pub(super) fields: Vec<Field>,
    pub(super) parent: Option<String>,
}

impl TryFrom<&xmltree::XMLNode> for ObjectType {
    type Error = TypeParseError;

    fn try_from(value: &xmltree::XMLNode) -> Result<Self, Self::Error> {
        match value {
            xmltree::XMLNode::Element(xmltree::Element {
                namespace: Some(namespace),
                name,
                attributes,
                children,
                ..
            }) if namespace == XML_SCHEMA_NS && name == "complexType" => {
                let name = attributes
                    .get("name")
                    .ok_or(TypeParseError::MissingName)?
                    .clone();
                let annotation = children
                    .iter()
                    .filter_map(|c| Annotation::try_from(c).ok())
                    .next()
                    .ok_or(TypeParseError::MissingAnnotation)?;
                let mut fields = Vec::new();
                let mut parent = None;
                fields.extend(children.iter().flat_map(Field::try_from));
                for child in children {
                    match child {
                        xmltree::XMLNode::Element(xmltree::Element {
                            namespace: Some(namespace),
                            name,
                            children,
                            ..
                        }) if namespace == XML_SCHEMA_NS && name == "sequence" => {
                            fields.extend(children.iter().flat_map(Field::try_from));
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
                                        parent = attributes.get("base").cloned();
                                        fields.extend(children.iter().flat_map(Field::try_from));
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
                                                    fields.extend(
                                                        children.iter().flat_map(Field::try_from),
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
                    parent,
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
                required: c.fields.iter().map(|s| s.name.clone()).collect(),
                ..Default::default()
            }));
        let schema_data = openapiv3::SchemaData {
            deprecated: c.annotation.deprecated,
            title: Some(c.name.clone()),
            description: Some(c.annotation.description.clone()),
            ..Default::default()
        };
        match &c.parent {
            None => openapiv3::Schema {
                schema_data,
                schema_kind,
            },
            Some(reference) => openapiv3::Schema {
                schema_data,
                schema_kind: openapiv3::SchemaKind::AllOf {
                    all_of: vec![
                        openapiv3::ReferenceOr::Reference {
                            reference: format!("#/components/schemas/{}", reference),
                        },
                        openapiv3::ReferenceOr::Item(openapiv3::Schema {
                            schema_kind,
                            schema_data: Default::default(),
                        }),
                    ],
                },
            },
        }
    }
}
