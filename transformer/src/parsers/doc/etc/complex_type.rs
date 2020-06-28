use crate::parsers::doc::etc::annotation::Annotation;
#[cfg(test)]
use crate::parsers::doc::etc::annotation::Modifiable;
#[cfg(test)]
use crate::parsers::doc::etc::sequence_element::Occurrences;
use crate::parsers::doc::etc::sequence_element::SequenceElement;
use crate::parsers::doc::etc::XML_SCHEMA_NS;
#[cfg(test)]
use serde_json::json;
use std::convert::TryFrom;
use thiserror::Error;

#[derive(Debug, PartialEq)]
pub(super) struct ComplexType {
    pub(super) annotation: Annotation,
    pub(super) name: String,
    pub(super) sequence_elements: Vec<SequenceElement>,
    pub(super) parent: Option<String>,
}

#[derive(Error, Debug, PartialEq)]
pub enum ComplexTypeParseError {
    #[error("not a complex type node")]
    NotComplexTypeNode,
    #[error("missing name attribute")]
    MissingName,
    #[error("missing annotation element")]
    MissingAnnotation,
}

impl TryFrom<&xmltree::XMLNode> for ComplexType {
    type Error = ComplexTypeParseError;

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
                    .ok_or(ComplexTypeParseError::MissingName)?
                    .clone();
                let annotation = children
                    .iter()
                    .filter_map(|c| Annotation::try_from(c).ok())
                    .next()
                    .ok_or(ComplexTypeParseError::MissingAnnotation)?;
                let mut sequence_elements = Vec::new();
                let mut parent = None;
                for child in children {
                    match child {
                        xmltree::XMLNode::Element(xmltree::Element {
                            namespace: Some(namespace),
                            name,
                            children,
                            ..
                        }) if namespace == XML_SCHEMA_NS && name == "sequence" => {
                            sequence_elements
                                .extend(children.iter().flat_map(SequenceElement::try_from));
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
                                        sequence_elements.extend(
                                            children.iter().flat_map(SequenceElement::try_from),
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
                                                    sequence_elements.extend(
                                                        children
                                                            .iter()
                                                            .flat_map(SequenceElement::try_from),
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
                Ok(ComplexType {
                    name,
                    annotation,
                    sequence_elements,
                    parent,
                })
            }
            _ => Err(ComplexTypeParseError::NotComplexTypeNode),
        }
    }
}

impl From<&ComplexType> for openapiv3::Schema {
    fn from(c: &ComplexType) -> Self {
        let schema_kind =
            openapiv3::SchemaKind::Type(openapiv3::Type::Object(openapiv3::ObjectType {
                properties: c
                    .sequence_elements
                    .iter()
                    .map(|s| {
                        (
                            s.name.clone(),
                            openapiv3::ReferenceOr::boxed_item(openapiv3::Schema::from(s)),
                        )
                    })
                    .collect(),
                additional_properties: Some(openapiv3::AdditionalProperties::Any(false)),
                required: c.sequence_elements.iter().map(|s| s.name.clone()).collect(),
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

#[test]
fn parse_base_type_test() {
    let xml: &[u8] = br#"
    <xs:complexType xmlns:xs="http://www.w3.org/2001/XMLSchema" abstract="true" name="BaseType">
        <xs:annotation>
            <xs:documentation source="since">0.9</xs:documentation>
            <xs:documentation xml:lang="en">
                A base abstract type for all the types.
            </xs:documentation>
        </xs:annotation>

        <xs:sequence>
            <xs:element name="BaseField" type="xs:string" minOccurs="0">
                <xs:annotation>
                    <xs:documentation source="modifiable">always</xs:documentation>
                    <xs:documentation xml:lang="en">
                        A base field for the base type
                    </xs:documentation>
                    <xs:documentation source="required">false</xs:documentation>
                </xs:annotation>
            </xs:element>
        </xs:sequence>
    </xs:complexType>
    "#;
    let tree = xmltree::Element::parse(xml).unwrap();
    assert_eq!(
        ComplexType::try_from(&xmltree::XMLNode::Element(tree)),
        Ok(ComplexType {
            annotation: Annotation {
                description: "A base abstract type for all the types.".to_owned(),
                required: None,
                deprecated: false,
                modifiable: None,
                content_type: None
            },
            name: "BaseType".to_owned(),
            sequence_elements: vec![SequenceElement {
                name: "baseField".to_owned(),
                r#type: "xs:string".to_owned(),
                occurrences: Occurrences::Optional,
                annotation: Some(Annotation {
                    description: "A base field for the base type".to_owned(),
                    required: Some(false),
                    deprecated: false,
                    modifiable: Some(Modifiable::Always),
                    content_type: None
                })
            }],
            parent: None
        })
    );
}

#[test]
fn parse_type_wth_parent_test() {
    let xml: &[u8] = br#"
    <xs:complexType xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:meta="http://www.vmware.com/vcloud/meta" name="TestType">
        <xs:annotation>
            <xs:appinfo>
                <meta:content-type>application/vnd.ccouzens.test</meta:content-type>
            </xs:appinfo>
            <xs:documentation source="since">0.9</xs:documentation>
            <xs:documentation xml:lang="en">
                A simple type to test the parser
            </xs:documentation>
        </xs:annotation>
        <xs:complexContent>
            <xs:extension base="BaseType">
                <xs:sequence>
                    <xs:element name="OptionalString" type="xs:string" minOccurs="0">
                        <xs:annotation>
                            <xs:documentation source="modifiable">always</xs:documentation>
                            <xs:documentation xml:lang="en">
                                String that may or may not be here
                            </xs:documentation>
                            <xs:documentation source="required">false</xs:documentation>
                        </xs:annotation>
                    </xs:element>
                    <xs:element name="RequiredString" type="xs:string" minOccurs="1">
                        <xs:annotation>
                            <xs:documentation source="modifiable">always</xs:documentation>
                            <xs:documentation xml:lang="en">
                                String that will be here
                            </xs:documentation>
                            <xs:documentation source="required">true</xs:documentation>
                        </xs:annotation>
                    </xs:element>
                </xs:sequence>
            </xs:extension>
        </xs:complexContent>
    </xs:complexType>
    "#;
    let tree = xmltree::Element::parse(xml).unwrap();
    assert_eq!(
        ComplexType::try_from(&xmltree::XMLNode::Element(tree)),
        Ok(ComplexType {
            annotation: Annotation {
                description: "A simple type to test the parser".to_owned(),
                required: None,
                deprecated: false,
                modifiable: None,
                content_type: Some("application/vnd.ccouzens.test".to_owned())
            },
            name: "TestType".to_owned(),
            sequence_elements: vec![
                SequenceElement {
                    annotation: Some(Annotation {
                        description: "String that may or may not be here".to_owned(),
                        required: Some(false),
                        deprecated: false,
                        modifiable: Some(Modifiable::Always),
                        content_type: None
                    }),
                    name: "optionalString".to_owned(),
                    r#type: "xs:string".to_owned(),
                    occurrences: Occurrences::Optional
                },
                SequenceElement {
                    annotation: Some(Annotation {
                        description: "String that will be here".to_owned(),
                        required: Some(true),
                        deprecated: false,
                        modifiable: Some(Modifiable::Always),
                        content_type: None
                    }),
                    name: "requiredString".to_string(),
                    r#type: "xs:string".to_owned(),
                    occurrences: Occurrences::One
                }
            ],
            parent: Some("BaseType".to_owned())
        })
    );
}

#[test]
fn parse_type_that_is_attribute_test() {
    let xml: &[u8] = br#"
    <xs:complexType xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:meta="http://www.vmware.com/vcloud/meta" name="TestType">
        <xs:annotation>
            <xs:appinfo>
                <meta:content-type>application/vnd.ccouzens.test</meta:content-type>
            </xs:appinfo>
            <xs:documentation source="since">0.9</xs:documentation>
            <xs:documentation xml:lang="en">
                A simple type to test the parser
            </xs:documentation>
        </xs:annotation>
        <xs:complexContent>
            <xs:extension base="BaseType">
                <xs:attribute name="requiredAttribute" type="xs:string" use="required">
                    <xs:annotation>
                        <xs:documentation source="modifiable">none</xs:documentation>
                        <xs:documentation>
                            A field that comes from an attribute.
                        </xs:documentation>
                        <xs:documentation source="required">true</xs:documentation>
                    </xs:annotation>
                </xs:attribute>
            </xs:extension>
        </xs:complexContent>
    </xs:complexType>
    "#;
    let tree = xmltree::Element::parse(xml).unwrap();
    assert_eq!(
        ComplexType::try_from(&xmltree::XMLNode::Element(tree)),
        Ok(ComplexType {
            annotation: Annotation {
                description: "A simple type to test the parser".to_owned(),
                required: None,
                deprecated: false,
                modifiable: None,
                content_type: Some("application/vnd.ccouzens.test".to_owned())
            },
            name: "TestType".to_owned(),
            sequence_elements: vec![SequenceElement {
                annotation: Some(Annotation {
                    description: "A field that comes from an attribute.".to_owned(),
                    required: Some(true),
                    deprecated: false,
                    modifiable: Some(Modifiable::None),
                    content_type: None
                }),
                name: "requiredAttribute".to_owned(),
                r#type: "xs:string".to_owned(),
                occurrences: Occurrences::One
            }],
            parent: Some("BaseType".to_owned())
        })
    );
}

#[test]
fn base_type_into_schema_test() {
    let xml: &[u8] = br#"
    <xs:complexType xmlns:xs="http://www.w3.org/2001/XMLSchema" abstract="true" name="BaseType">
        <xs:annotation>
            <xs:documentation source="since">0.9</xs:documentation>
            <xs:documentation xml:lang="en">
                A base abstract type for all the types.
            </xs:documentation>
        </xs:annotation>

        <xs:sequence>
            <xs:element name="BaseField" type="xs:string" minOccurs="0">
                <xs:annotation>
                    <xs:documentation source="modifiable">always</xs:documentation>
                    <xs:documentation xml:lang="en">
                        A base field for the base type
                    </xs:documentation>
                    <xs:documentation source="required">false</xs:documentation>
                </xs:annotation>
            </xs:element>
        </xs:sequence>
    </xs:complexType>
    "#;
    let tree = xmltree::Element::parse(xml).unwrap();
    let c = ComplexType::try_from(&xmltree::XMLNode::Element(tree)).unwrap();
    let value = openapiv3::Schema::from(&c);
    assert_eq!(
        serde_json::to_value(value).unwrap(),
        json!({
            "additionalProperties": false,
            "description": "A base abstract type for all the types.",
            "properties": {
                "baseField": {
                    "description": "A base field for the base type",
                    "nullable": true,
                    "type": "string"
                }
            },
            "required": ["baseField"],
            "title": "BaseType",
            "type": "object"
        })
    );
}

#[test]
fn parent_type_into_schema_test() {
    let xml: &[u8] = br#"
    <xs:complexType xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:meta="http://www.vmware.com/vcloud/meta" name="TestType">
        <xs:annotation>
            <xs:appinfo>
                <meta:content-type>application/vnd.ccouzens.test</meta:content-type>
            </xs:appinfo>
            <xs:documentation source="since">0.9</xs:documentation>
            <xs:documentation xml:lang="en">
                A simple type to test the parser
            </xs:documentation>
        </xs:annotation>
        <xs:complexContent>
            <xs:extension base="BaseType">
                <xs:sequence>
                    <xs:element name="OptionalString" type="xs:string" minOccurs="0">
                        <xs:annotation>
                            <xs:documentation source="modifiable">always</xs:documentation>
                            <xs:documentation xml:lang="en">
                                String that may or may not be here
                            </xs:documentation>
                            <xs:documentation source="required">false</xs:documentation>
                        </xs:annotation>
                    </xs:element>
                    <xs:element name="RequiredString" type="xs:string" minOccurs="1">
                        <xs:annotation>
                            <xs:documentation source="modifiable">always</xs:documentation>
                            <xs:documentation xml:lang="en">
                                String that will be here
                            </xs:documentation>
                            <xs:documentation source="required">true</xs:documentation>
                        </xs:annotation>
                    </xs:element>
                </xs:sequence>
            </xs:extension>
        </xs:complexContent>
    </xs:complexType>
    "#;
    let tree = xmltree::Element::parse(xml).unwrap();
    let c = ComplexType::try_from(&xmltree::XMLNode::Element(tree)).unwrap();
    let value = openapiv3::Schema::from(&c);
    assert_eq!(
        serde_json::to_value(value).unwrap(),
        json!({
            "description": "A simple type to test the parser",
            "allOf": [
                {
                    "$ref": "#/components/schemas/BaseType"
                },
                {
                  "additionalProperties": false,
                  "properties": {
                    "optionalString": {
                        "description": "String that may or may not be here",
                        "nullable": true,
                        "type": "string"
                    },
                    "requiredString": {
                        "description": "String that will be here",
                        "type": "string"
                    }
                  },
                  "required": ["optionalString", "requiredString"],
                  "type": "object"
              }
            ],
            "title": "TestType",
        })
    );
}
