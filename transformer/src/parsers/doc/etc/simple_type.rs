use crate::parsers::doc::etc::annotation::Annotation;
use crate::parsers::doc::etc::primitive_type::PrimitiveType;
use crate::parsers::doc::etc::primitive_type::RestrictedPrimitiveType;
use crate::parsers::doc::etc::r#type::TypeParseError;
use crate::parsers::doc::etc::XML_SCHEMA_NS;
use std::convert::TryFrom;

#[derive(Debug, PartialEq, Clone)]
pub(super) struct SimpleType {
    pub(super) annotation: Option<Annotation>,
    pub(super) name: Option<String>,
    pub(super) pattern: Option<String>,
    pub(super) list: bool,
    pub(super) parent: PrimitiveType,
    pub(super) enumeration: Vec<Option<String>>,
    pub(super) min_inclusive: Option<String>,
}

impl TryFrom<(&xmltree::XMLNode, &str)> for SimpleType {
    type Error = TypeParseError;

    fn try_from((xml, schema_namespace): (&xmltree::XMLNode, &str)) -> Result<Self, Self::Error> {
        match xml {
            xmltree::XMLNode::Element(xmltree::Element {
                namespace: Some(namespace),
                name,
                attributes,
                children,
                ..
            }) if namespace == XML_SCHEMA_NS && name == "simpleType" => {
                let name = attributes
                    .get("name")
                    .map(|n| format!("{}_{}", schema_namespace, n));
                let annotation = children
                    .iter()
                    .filter_map(|c| Annotation::try_from(c).ok())
                    .next();
                for child in children {
                    match child {
                        xmltree::XMLNode::Element(xmltree::Element {
                            namespace: Some(namespace),
                            name: node_name,
                            attributes,
                            ..
                        }) if namespace == XML_SCHEMA_NS && node_name == "list" => {
                            let parent = attributes
                                .get("itemType")
                                .ok_or(TypeParseError::MissingItemTypeValue)?;
                            return Ok(Self {
                                annotation,
                                name,
                                enumeration: Vec::new(),
                                list: true,
                                min_inclusive: None,
                                parent: parent.parse()?,
                                pattern: None,
                            });
                        }
                        xmltree::XMLNode::Element(xmltree::Element {
                            namespace: Some(namespace),
                            name: node_name,
                            attributes,
                            children,
                            ..
                        }) if namespace == XML_SCHEMA_NS && node_name == "restriction" => {
                            let parent =
                                attributes.get("base").ok_or(TypeParseError::MissingBase)?;
                            let pattern = children
                                .iter()
                                .filter_map(|child| match child {
                                    xmltree::XMLNode::Element(xmltree::Element {
                                        namespace: Some(namespace),
                                        name,
                                        attributes,
                                        ..
                                    }) if namespace == XML_SCHEMA_NS && name == "pattern" => {
                                        attributes.get("value").cloned()
                                    }
                                    _ => None,
                                })
                                .next();
                            let min_inclusive = children
                                .iter()
                                .filter_map(|child| match child {
                                    xmltree::XMLNode::Element(xmltree::Element {
                                        namespace: Some(namespace),
                                        name,
                                        attributes,
                                        ..
                                    }) if namespace == XML_SCHEMA_NS && name == "minInclusive" => {
                                        attributes.get("value").cloned()
                                    }
                                    _ => None,
                                })
                                .next();

                            let enumeration = children
                                .iter()
                                .filter_map(|child| match child {
                                    xmltree::XMLNode::Element(xmltree::Element {
                                        namespace: Some(namespace),
                                        name,
                                        attributes,
                                        ..
                                    }) if namespace == XML_SCHEMA_NS && name == "enumeration" => {
                                        Some(attributes.get("value").cloned())
                                    }
                                    _ => None,
                                })
                                .collect();
                            return Ok(Self {
                                annotation,
                                name,
                                enumeration,
                                list: false,
                                min_inclusive,
                                parent: parent.parse()?,
                                pattern,
                            });
                        }
                        _ => {}
                    }
                }
                Err(TypeParseError::NotTypeNode)
            }
            _ => Err(TypeParseError::NotTypeNode),
        }
    }
}

pub(super) fn str_to_simple_type_or_reference(
    namespace: &str,
    type_name: &str,
) -> openapiv3::ReferenceOr<SimpleType> {
    match type_name.parse::<PrimitiveType>() {
        Err(_) => openapiv3::ReferenceOr::Reference {
            reference: if type_name.contains(':') {
                type_name.replacen(':', "_", 1)
            } else {
                format!("{}_{}", namespace, type_name)
            },
        },
        Ok(p) => openapiv3::ReferenceOr::Item(SimpleType {
            annotation: None,
            enumeration: Vec::new(),
            list: false,
            min_inclusive: None,
            name: None,
            parent: p,
            pattern: None,
        }),
    }
}

impl From<&SimpleType> for openapiv3::Schema {
    fn from(t: &SimpleType) -> Self {
        let schema_data = openapiv3::SchemaData {
            deprecated: t.annotation.as_ref().map(|a| a.deprecated).unwrap_or(false),
            title: t.name.clone(),
            description: t.annotation.as_ref().and_then(|a| a.description.clone()),
            ..Default::default()
        };

        let schema_kind =
            openapiv3::SchemaKind::Type(openapiv3::Type::from(&RestrictedPrimitiveType {
                r#type: t.parent,
                enumeration: &t.enumeration,
                min_inclusive: &t.min_inclusive,
                pattern: &t.pattern,
            }));
        if t.list {
            Self {
                schema_data,
                schema_kind: openapiv3::SchemaKind::Type(openapiv3::Type::Array(
                    openapiv3::ArrayType {
                        items: Some(openapiv3::ReferenceOr::boxed_item(openapiv3::Schema {
                            schema_kind,
                            schema_data: Default::default(),
                        })),
                        max_items: None,
                        min_items: None,
                        unique_items: false,
                    },
                )),
            }
        } else {
            Self {
                schema_data,
                schema_kind,
            }
        }
    }
}
