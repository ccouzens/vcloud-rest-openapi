use crate::parsers::doc::etc::annotation::Annotation;
use crate::parsers::doc::etc::annotation::Modifiable;
use crate::parsers::doc::etc::XML_SCHEMA_NS;
#[cfg(test)]
use serde_json::json;
use std::convert::TryFrom;
use thiserror::Error;

#[derive(Debug, PartialEq, Clone, Copy)]
pub(super) enum Occurrences {
    Optional,
    One,
    Array,
}

#[derive(Debug, PartialEq)]
pub(super) struct SequenceElement {
    pub(super) annotation: Option<Annotation>,
    pub(super) name: String,
    pub(super) r#type: String,
    pub(super) occurrences: Occurrences,
}

#[derive(Error, Debug, PartialEq)]
pub enum SequenceElementParseError {
    #[error("missing name attribute")]
    MissingName,
    #[error("missing type attribute")]
    MissingType,
    #[error("not a sequence element node")]
    NotSequenceElementNode,
}

impl TryFrom<&xmltree::XMLNode> for SequenceElement {
    type Error = SequenceElementParseError;

    fn try_from(value: &xmltree::XMLNode) -> Result<Self, Self::Error> {
        match value {
            xmltree::XMLNode::Element(xmltree::Element {
                namespace: Some(namespace),
                name,
                attributes,
                children,
                ..
            }) if namespace == XML_SCHEMA_NS && name == "element" => {
                // Name comes from the attribute name.
                // In the XML the fields start with a uppercase letter.
                // But in the JSON, the first letter is lowercase.
                let name = attributes
                    .get("name")
                    .ok_or(SequenceElementParseError::MissingName)?
                    .chars()
                    .enumerate()
                    .map(|(i, c)| if i == 0 { c.to_ascii_lowercase() } else { c })
                    .collect();
                let r#type = attributes
                    .get("type")
                    .ok_or(SequenceElementParseError::MissingType)?
                    .to_owned();
                let occurrences = match (
                    attributes
                        .get("minOccurs")
                        .map(String::as_str)
                        .unwrap_or("1"),
                    attributes
                        .get("maxOccurs")
                        .map(String::as_str)
                        .unwrap_or("1"),
                ) {
                    (_, "unbounded") => Occurrences::Array,
                    ("0", _) => Occurrences::Optional,
                    _ => Occurrences::One,
                };
                let annotation = children.iter().flat_map(Annotation::try_from).next();
                Ok(SequenceElement {
                    annotation,
                    name,
                    r#type,
                    occurrences,
                })
            }
            _ => Err(SequenceElementParseError::NotSequenceElementNode),
        }
    }
}

impl From<&SequenceElement> for openapiv3::Schema {
    fn from(s: &SequenceElement) -> Self {
        let reference_or_schema_type = match s.r#type.as_ref() {
            "xs:anyURI" => {
                openapiv3::ReferenceOr::Item(openapiv3::Type::String(openapiv3::StringType {
                    format: openapiv3::VariantOrUnknownOrEmpty::Unknown("uri".to_owned()),
                    ..Default::default()
                }))
            }
            "xs:boolean" => openapiv3::ReferenceOr::Item(openapiv3::Type::Boolean {}),
            "xs:double" => {
                openapiv3::ReferenceOr::Item(openapiv3::Type::Number(openapiv3::NumberType {
                    format: openapiv3::VariantOrUnknownOrEmpty::Item(
                        openapiv3::NumberFormat::Double,
                    ),
                    ..Default::default()
                }))
            }
            "xs:int" => {
                openapiv3::ReferenceOr::Item(openapiv3::Type::Integer(openapiv3::IntegerType {
                    format: openapiv3::VariantOrUnknownOrEmpty::Item(
                        openapiv3::IntegerFormat::Int32,
                    ),
                    ..Default::default()
                }))
            }
            "xs:string" => {
                openapiv3::ReferenceOr::Item(openapiv3::Type::String(Default::default()))
            }
            other => openapiv3::ReferenceOr::Reference {
                reference: format!("#/components/schemas/{}", other),
            },
        };

        openapiv3::Schema {
            schema_data: openapiv3::SchemaData {
                nullable: s.occurrences == Occurrences::Optional,
                read_only: s.annotation.as_ref().and_then(|a| a.modifiable)
                    == Some(Modifiable::None),
                deprecated: s.annotation.as_ref().map(|a| a.deprecated) == Some(true),
                description: s.annotation.as_ref().map(|a| a.description.clone()),
                ..Default::default()
            },
            schema_kind: match (s.occurrences, reference_or_schema_type) {
                (Occurrences::Array, openapiv3::ReferenceOr::Item(schema_type)) => {
                    openapiv3::SchemaKind::Type(openapiv3::Type::Array(openapiv3::ArrayType {
                        items: openapiv3::ReferenceOr::boxed_item(openapiv3::Schema {
                            schema_data: Default::default(),
                            schema_kind: openapiv3::SchemaKind::Type(schema_type),
                        }),
                        min_items: None,
                        max_items: None,
                        unique_items: false,
                    }))
                }

                (Occurrences::Array, openapiv3::ReferenceOr::Reference { reference }) => {
                    openapiv3::SchemaKind::Type(openapiv3::Type::Array(openapiv3::ArrayType {
                        items: openapiv3::ReferenceOr::Reference { reference },
                        min_items: None,
                        max_items: None,
                        unique_items: false,
                    }))
                }

                (_, openapiv3::ReferenceOr::Item(schema_type)) => {
                    openapiv3::SchemaKind::Type(schema_type)
                }

                (_, openapiv3::ReferenceOr::Reference { reference }) => {
                    openapiv3::SchemaKind::AllOf {
                        all_of: vec![openapiv3::ReferenceOr::Reference { reference }],
                    }
                }
            },
        }
    }
}

#[test]
fn test_parse_sequence_element_optional() {
    let xml: &[u8] = br#"
    <xs:element xmlns:xs="http://www.w3.org/2001/XMLSchema" name="BaseField" type="xs:string" minOccurs="0">
        <xs:annotation>
            <xs:documentation source="modifiable">always</xs:documentation>
            <xs:documentation xml:lang="en">
                A base field for the base type
            </xs:documentation>
            <xs:documentation source="required">false</xs:documentation>
        </xs:annotation>
    </xs:element>
    "#;
    let tree = xmltree::Element::parse(xml).unwrap();
    assert_eq!(
        SequenceElement::try_from(&xmltree::XMLNode::Element(tree)),
        Ok(SequenceElement {
            annotation: Some(Annotation {
                description: "A base field for the base type".to_owned(),
                required: Some(false),
                deprecated: false,
                modifiable: Some(Modifiable::Always),
                content_type: None
            }),
            name: "baseField".to_owned(),
            r#type: "xs:string".to_owned(),
            occurrences: Occurrences::Optional
        })
    );
}

#[test]
fn test_parse_sequence_element_array() {
    let xml: &[u8] = br#"
    <xs:element xmlns:xs="http://www.w3.org/2001/XMLSchema" name="BaseField" type="xs:int" minOccurs="0" maxOccurs="unbounded">
        <xs:annotation>
            <xs:documentation source="modifiable">none</xs:documentation>
            <xs:documentation xml:lang="en">
                A field that could be repeated many times in the &lt;code&gt;XML&lt;/code&gt;.
            </xs:documentation>
            <xs:documentation source="required">false</xs:documentation>
        </xs:annotation>
    </xs:element>
    "#;
    let tree = xmltree::Element::parse(xml).unwrap();
    assert_eq!(
        SequenceElement::try_from(&xmltree::XMLNode::Element(tree)),
        Ok(SequenceElement {
            annotation: Some(Annotation {
                description: "A field that could be repeated many times in the `XML`.".to_owned(),
                required: Some(false),
                deprecated: false,
                modifiable: Some(Modifiable::None),
                content_type: None
            }),
            name: "baseField".to_owned(),
            r#type: "xs:int".to_owned(),
            occurrences: Occurrences::Array
        })
    );
}

#[test]
fn test_parse_sequence_element_exactly_one() {
    let xml: &[u8] = br#"
    <xs:element xmlns:xs="http://www.w3.org/2001/XMLSchema" name="BaseField" type="xs:boolean">
        <xs:annotation>
            <xs:documentation source="modifiable">none</xs:documentation>
            <xs:documentation xml:lang="en">
                A field that appears precisely once in the &lt;code&gt;XML&lt;/code&gt;.
            </xs:documentation>
            <xs:documentation source="required">true</xs:documentation>
        </xs:annotation>
    </xs:element>
    "#;
    let tree = xmltree::Element::parse(xml).unwrap();
    assert_eq!(
        SequenceElement::try_from(&xmltree::XMLNode::Element(tree)),
        Ok(SequenceElement {
            annotation: Some(Annotation {
                description: "A field that appears precisely once in the `XML`.".to_owned(),
                required: Some(true),
                deprecated: false,
                modifiable: Some(Modifiable::None),
                content_type: None
            }),
            name: "baseField".to_owned(),
            r#type: "xs:boolean".to_owned(),
            occurrences: Occurrences::One
        })
    );
}

#[test]
fn test_sequence_element_optional_into_schema() {
    let xml: &[u8] = br#"
    <xs:element xmlns:xs="http://www.w3.org/2001/XMLSchema" name="BaseField" type="xs:string" minOccurs="0">
        <xs:annotation>
            <xs:documentation source="modifiable">always</xs:documentation>
            <xs:documentation xml:lang="en">
                A base field for the base type
            </xs:documentation>
            <xs:documentation source="required">false</xs:documentation>
        </xs:annotation>
    </xs:element>
    "#;
    let tree = xmltree::Element::parse(xml).unwrap();
    let s = SequenceElement::try_from(&xmltree::XMLNode::Element(tree)).unwrap();
    let value = openapiv3::Schema::from(&s);
    assert_eq!(
        serde_json::to_value(value).unwrap(),
        json!({
            "description": "A base field for the base type",
            "nullable": true,
            "type": "string"
        })
    );
}

#[test]
fn test_sequence_element_array_into_schema() {
    let xml: &[u8] = br#"
    <xs:element xmlns:xs="http://www.w3.org/2001/XMLSchema" name="BaseField" type="xs:int" minOccurs="0" maxOccurs="unbounded">
        <xs:annotation>
            <xs:documentation source="modifiable">none</xs:documentation>
            <xs:documentation xml:lang="en">
                A field that could be repeated many times in the &lt;code&gt;XML&lt;/code&gt;.
            </xs:documentation>
            <xs:documentation source="required">false</xs:documentation>
        </xs:annotation>
    </xs:element>
    "#;
    let tree = xmltree::Element::parse(xml).unwrap();
    let s = SequenceElement::try_from(&xmltree::XMLNode::Element(tree)).unwrap();
    let value = openapiv3::Schema::from(&s);
    assert_eq!(
        serde_json::to_value(value).unwrap(),
        json!({
            "description": "A field that could be repeated many times in the `XML`.",
            "type": "array",
            "readOnly": true,
            "items": {
                "format": "int32",
                "type": "integer"
            }
        })
    );
}

#[test]
fn test_sequence_element_exactly_one_into_schema() {
    let xml: &[u8] = br#"
    <xs:element xmlns:xs="http://www.w3.org/2001/XMLSchema" name="BaseField" type="xs:boolean">
        <xs:annotation>
            <xs:documentation source="modifiable">none</xs:documentation>
            <xs:documentation xml:lang="en">
                A field that appears precisely once in the &lt;code&gt;XML&lt;/code&gt;.
            </xs:documentation>
            <xs:documentation source="required">true</xs:documentation>
        </xs:annotation>
    </xs:element>
    "#;
    let tree = xmltree::Element::parse(xml).unwrap();
    let s = SequenceElement::try_from(&xmltree::XMLNode::Element(tree)).unwrap();
    let value = openapiv3::Schema::from(&s);
    assert_eq!(
        serde_json::to_value(value).unwrap(),
        json!({
            "description": "A field that appears precisely once in the `XML`.",
            "type": "boolean",
            "readOnly": true,
        })
    );
}

#[test]
fn test_anyuri_into_schema() {
    let xml: &[u8] = br#"
    <xs:element xmlns:xs="http://www.w3.org/2001/XMLSchema" name="BaseField" type="xs:anyURI">
        <xs:annotation>
            <xs:documentation source="modifiable">none</xs:documentation>
            <xs:documentation xml:lang="en">
                A field that is meant to represent a URL.
            </xs:documentation>
            <xs:documentation source="required">true</xs:documentation>
        </xs:annotation>
    </xs:element>
    "#;
    let tree = xmltree::Element::parse(xml).unwrap();
    let s = SequenceElement::try_from(&xmltree::XMLNode::Element(tree)).unwrap();
    let value = openapiv3::Schema::from(&s);
    assert_eq!(
        serde_json::to_value(value).unwrap(),
        json!({
            "description": "A field that is meant to represent a URL.",
            "format": "uri",
            "type": "string",
            "readOnly": true,
        })
    );
}

#[test]
fn test_double_into_schema() {
    let xml: &[u8] = br#"
    <xs:element xmlns:xs="http://www.w3.org/2001/XMLSchema" name="BaseField" type="xs:double">
        <xs:annotation>
            <xs:documentation source="modifiable">none</xs:documentation>
            <xs:documentation xml:lang="en">
                A field that represents a double precision float
            </xs:documentation>
            <xs:documentation source="required">true</xs:documentation>
        </xs:annotation>
    </xs:element>
    "#;
    let tree = xmltree::Element::parse(xml).unwrap();
    let s = SequenceElement::try_from(&xmltree::XMLNode::Element(tree)).unwrap();
    let value = openapiv3::Schema::from(&s);
    assert_eq!(
        serde_json::to_value(value).unwrap(),
        json!({
            "description": "A field that represents a double precision float",
            "format": "double",
            "type": "number",
            "readOnly": true,
        })
    );
}
