use crate::parsers::doc::etc::annotation::Annotation;
use crate::parsers::doc::etc::primitive_type::RestrictedPrimitiveType;
use crate::parsers::doc::etc::simple_type::str_to_simple_type_or_reference;
use crate::parsers::doc::etc::simple_type::SimpleType;
use crate::parsers::doc::etc::XML_SCHEMA_NS;
#[cfg(test)]
use serde_json::json;

use std::convert::TryFrom;
use thiserror::Error;
use xmltree::XMLNode;

#[derive(Debug, PartialEq, Clone, Copy)]
pub(super) enum Occurrences {
    Optional,
    One,
    Array,
}

#[derive(Debug, PartialEq)]
pub(super) struct Field {
    pub(super) annotation: Option<Annotation>,
    pub(super) name: String,
    pub(super) r#type: openapiv3::ReferenceOr<SimpleType>,
    pub(super) occurrences: Occurrences,
}

#[derive(Error, Debug, PartialEq)]
pub enum FieldParseError {
    #[error("missing name attribute")]
    MissingName,
    #[error("missing type attribute")]
    MissingType,
    #[error("not a sequence element node")]
    NotFieldNode,
    #[error("this field is marked as removed")]
    Removed,
}

impl
    TryFrom<(
        Option<&str>,
        &xmltree::XMLNode,
        &Vec<(Option<&str>, xmltree::XMLNode)>,
    )> for Field
{
    type Error = FieldParseError;

    fn try_from(
        (ns, xml, types): (
            Option<&str>,
            &xmltree::XMLNode,
            &Vec<(Option<&str>, xmltree::XMLNode)>,
        ),
    ) -> Result<Self, Self::Error> {
        match xml {
            xmltree::XMLNode::Element(xmltree::Element {
                namespace: Some(namespace),
                name,
                attributes,
                children,
                ..
            }) if namespace == XML_SCHEMA_NS && name == "element" => {
                match attributes
                    .get("ref")
                    .and_then(|type_name| {
                        types.iter().find_map(|(_, xml)| {
                            xml.as_element()
                                .and_then(|e| {
                                    e.children.iter().find(|&child| match child {
                                        xmltree::XMLNode::Element(xmltree::Element {
                                            attributes,
                                            ..
                                        }) => attributes.get("name").map_or(false, |name| {
                                            match type_name.split_once(':') {
                                                Some((ns, tn)) if ns.eq("class") && tn.eq(name) => {
                                                    true
                                                }
                                                Some((ns, tn)) if tn.eq(name) => {
                                                    attributes.get("type").map_or(false, |t| {
                                                        t.split_once(':')
                                                            .map_or(false, |(tns, _)| ns == tns)
                                                    })
                                                }
                                                Some((_, _)) => false,
                                                None => type_name.eq(name),
                                            }
                                        }),
                                        _ => false,
                                    })
                                })
                                .and_then(|xml| match xml {
                                    xmltree::XMLNode::Element(xmltree::Element {
                                        attributes,
                                        ..
                                    }) => attributes.get("name").map(|name| (name, xml)),
                                    _ => None,
                                })
                                .and_then(|(name, xml)| match xml {
                                    xmltree::XMLNode::Element(xmltree::Element {
                                        attributes,
                                        ..
                                    }) => SimpleType::try_from((ns, xml))
                                            .map(|s| openapiv3::ReferenceOr::Item(s))
                                            .ok()
                                            .or(attributes.get("type").map(|type_name| {
                                                str_to_simple_type_or_reference(ns, type_name, None)
                                            }))
                                            .map(|r#type| (name, r#type)),
                                    _ => None,
                                })
                        })
                    })
                    .or(children
                        .iter()
                        .flat_map(|xml| SimpleType::try_from((ns, xml)))
                        .next()
                        .map(|s| openapiv3::ReferenceOr::Item(s))
                        .or(attributes
                            .get("type")
                            .map(|type_name| str_to_simple_type_or_reference(ns, type_name, None)))
                        .and_then(|r#type| attributes.get("name").map(|name| (name, r#type))))
                    .and_then(|(name, ref r#type)| {
                        children
                            .iter()
                            .flat_map(Annotation::try_from)
                            .next()
                            .map(|annotation| match annotation {
                                Annotation { removed: true, .. } => Err(FieldParseError::Removed),
                                _ => Ok(Field {
                                    annotation: Some(annotation),
                                    name: decapitalize(name),
                                    r#type: r#type.to_owned(),
                                    occurrences: get_occurrences(xml),
                                }),
                            })
                            .or(Some(Ok(Field {
                                annotation: None,
                                name: decapitalize(name),
                                r#type: r#type.to_owned(),
                                occurrences: get_occurrences(xml),
                            })))
                    }) {
                    Some(result) => result,
                    None => Err(FieldParseError::MissingType),
                }
            }
            xmltree::XMLNode::Element(xmltree::Element {
                namespace: Some(namespace),
                name,
                attributes,
                children,
                ..
            }) if namespace == XML_SCHEMA_NS && name == "attribute" => {
                let name = attributes
                    .get("name")
                    .map(|name| decapitalize(name))
                    .ok_or(FieldParseError::MissingName)?
                    .to_owned();
                let r#type = match children
                    .iter()
                    .flat_map(|xml| SimpleType::try_from((ns, xml)))
                    .next()
                {
                    Some(s) => openapiv3::ReferenceOr::Item(s),
                    None => {
                        let type_name =
                            attributes.get("type").ok_or(FieldParseError::MissingType)?;
                        str_to_simple_type_or_reference(ns, type_name, None)
                    }
                };
                let occurrences = match attributes.get("use").map(String::as_str) {
                    Some("required") => Occurrences::One,
                    _ => Occurrences::Optional,
                };
                let annotation = children.iter().flat_map(Annotation::try_from).next();
                Ok(Field {
                    annotation,
                    name,
                    r#type,
                    occurrences,
                })
            }
            _ => Err(FieldParseError::NotFieldNode),
        }
    }
}

impl From<&Field> for openapiv3::ReferenceOr<openapiv3::Type> {
    fn from(s: &Field) -> Self {
        match &s.r#type {
            openapiv3::ReferenceOr::Item(s) => {
                openapiv3::ReferenceOr::Item(openapiv3::Type::from(&RestrictedPrimitiveType {
                    r#type: s.parent,
                    enumeration: &s.enumeration,
                    min_inclusive: &s.min_inclusive,
                    pattern: &s.pattern,
                }))
            }
            openapiv3::ReferenceOr::Reference { reference } => openapiv3::ReferenceOr::Reference {
                reference: format!("#/components/schemas/{}", reference),
            },
        }
    }
}

impl From<&Field> for openapiv3::ReferenceOr<openapiv3::Schema> {
    fn from(s: &Field) -> Self {
        let reference_or_schema_type = openapiv3::ReferenceOr::from(s);
        let schema_data = openapiv3::SchemaData {
            nullable: false,
            read_only: false,
            deprecated: s.annotation.as_ref().map(|a| a.deprecated) == Some(true),
            description: s.annotation.as_ref().and_then(|a| a.description.clone()),
            ..Default::default()
        };
        match (s.occurrences, reference_or_schema_type) {
            (Occurrences::Array, openapiv3::ReferenceOr::Item(schema_type)) => {
                openapiv3::ReferenceOr::Item(openapiv3::Schema {
                    schema_data,
                    schema_kind: openapiv3::SchemaKind::Type(openapiv3::Type::Array(
                        openapiv3::ArrayType {
                            items: Some(openapiv3::ReferenceOr::boxed_item(openapiv3::Schema {
                                schema_data: Default::default(),
                                schema_kind: openapiv3::SchemaKind::Type(schema_type),
                            })),
                            min_items: None,
                            max_items: None,
                            unique_items: false,
                        },
                    )),
                })
            }

            (Occurrences::Array, openapiv3::ReferenceOr::Reference { reference }) => {
                openapiv3::ReferenceOr::Item(openapiv3::Schema {
                    schema_data,
                    schema_kind: openapiv3::SchemaKind::Type(openapiv3::Type::Array(
                        openapiv3::ArrayType {
                            items: Some(openapiv3::ReferenceOr::Reference { reference }),
                            min_items: None,
                            max_items: None,
                            unique_items: false,
                        },
                    )),
                })
            }

            (_, openapiv3::ReferenceOr::Item(schema_type)) => {
                openapiv3::ReferenceOr::Item(openapiv3::Schema {
                    schema_data,
                    schema_kind: openapiv3::SchemaKind::Type(schema_type),
                })
            }

            (_, openapiv3::ReferenceOr::Reference { reference }) => {
                openapiv3::ReferenceOr::Reference { reference }
            }
        }
    }
}

/// Decapitalizes the first character in s.
fn decapitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_lowercase().collect::<String>() + c.as_str(),
    }
}

/// Occurrence Constraints
fn get_occurrences(xml: &XMLNode) -> Occurrences {
    match xml {
        xmltree::XMLNode::Element(xmltree::Element { attributes, .. }) => match (
            attributes.get("minOccurs").map(String::as_str),
            attributes.get("maxOccurs").map(String::as_str),
        ) {
            (Some("1"), Some("1")) | (Some("1"), None) => Occurrences::One,
            (Some("0"), Some("1")) | (Some("0"), None) => Occurrences::Optional,
            (_, Some("unbounded")) | (Some(_), None) => Occurrences::Array,
            (Some(_), Some(max_occurs))
                if max_occurs
                    .parse::<u32>()
                    .map_or(false, |max_occurs| max_occurs > 1) =>
            {
                Occurrences::Array
            }
            _ => Occurrences::One,
        },
        _ => Occurrences::One,
    }
}

#[test]
fn test_parse_field_from_required_attribute() {
    let xml: &[u8] = br#"
    <xs:attribute xmlns:xs="http://www.w3.org/2001/XMLSchema" name="requiredAttribute" type="xs:string" use="required">
        <xs:annotation>
            <xs:documentation source="modifiable">none</xs:documentation>
            <xs:documentation>
                A field that comes from an attribute.
            </xs:documentation>
            <xs:documentation source="required">true</xs:documentation>
        </xs:annotation>
    </xs:attribute>
"#;
    let ns: Option<&str> = None;
    let tree = xmltree::Element::parse(xml).unwrap();
    let types = xmltree::Element::parse(xml).unwrap();
    let s = Field::try_from((
        ns,
        &xmltree::XMLNode::Element(tree),
        &vec![(ns, xmltree::XMLNode::Element(types))],
    ))
    .unwrap();
    let value: openapiv3::ReferenceOr<openapiv3::Schema> = openapiv3::ReferenceOr::from(&s);
    assert_eq!(
        serde_json::to_value(value).unwrap(),
        json!({
            "description": "A field that comes from an attribute.",
            "type": "string"
        })
    );
}

#[test]
fn test_parse_field_from_optional_attribute() {
    let xml: &[u8] = br#"
    <xs:attribute xmlns:xs="http://www.w3.org/2001/XMLSchema" name="optionalAttribute" type="xs:string">
        <xs:annotation>
            <xs:documentation source="modifiable">none</xs:documentation>
            <xs:documentation>
                A field that comes from an optional attribute.
            </xs:documentation>
            <xs:documentation source="required">false</xs:documentation>
        </xs:annotation>
    </xs:attribute>
"#;
    let ns: Option<&str> = None;
    let tree = xmltree::Element::parse(xml).unwrap();
    let types = xmltree::Element::parse(xml).unwrap();
    let s = Field::try_from((
        ns,
        &xmltree::XMLNode::Element(tree),
        &vec![(ns, xmltree::XMLNode::Element(types))],
    ))
    .unwrap();
let value: openapiv3::ReferenceOr<openapiv3::Schema> = openapiv3::ReferenceOr::from(&s);
    assert_eq!(
        serde_json::to_value(value).unwrap(),
        json!({
            "description": "A field that comes from an optional attribute.",
            "type": "string"
        })
    );
}

#[test]
fn test_field_optional_into_schema() {
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
    let ns: Option<&str> = None;
    let tree = xmltree::Element::parse(xml).unwrap();
    let types = xmltree::Element::parse(xml).unwrap();
    let s = Field::try_from((
        ns,
        &xmltree::XMLNode::Element(tree),
        &vec![(ns, xmltree::XMLNode::Element(types))],
    ))
    .unwrap();
let value: openapiv3::ReferenceOr<openapiv3::Schema> = openapiv3::ReferenceOr::from(&s);
    assert_eq!(
        serde_json::to_value(value).unwrap(),
        json!({
            "description": "A base field for the base type",
            "type": "string"
        })
    );
}

#[test]
fn test_field_array_into_schema() {
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
    let ns: Option<&str> = None;
    let tree = xmltree::Element::parse(xml).unwrap();
    let types = xmltree::Element::parse(xml).unwrap();
    let s = Field::try_from((
        ns,
        &xmltree::XMLNode::Element(tree),
        &vec![(ns, xmltree::XMLNode::Element(types))],
    ))
    .unwrap();
let value: openapiv3::ReferenceOr<openapiv3::Schema> = openapiv3::ReferenceOr::from(&s);
    assert_eq!(
        serde_json::to_value(value).unwrap(),
        json!({
            "description": "A field that could be repeated many times in the `XML`.",
            "type": "array",
            "items": {
                "format": "int32",
                "type": "integer"
            }
        })
    );
}

#[test]
fn test_field_exactly_one_into_schema() {
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
    let ns: Option<&str> = None;
    let tree = xmltree::Element::parse(xml).unwrap();
    let types = xmltree::Element::parse(xml).unwrap();
    let s = Field::try_from((
        ns,
        &xmltree::XMLNode::Element(tree),
        &vec![(ns, xmltree::XMLNode::Element(types))],
    ))
    .unwrap();
let value: openapiv3::ReferenceOr<openapiv3::Schema> = openapiv3::ReferenceOr::from(&s);
    assert_eq!(
        serde_json::to_value(value).unwrap(),
        json!({
            "description": "A field that appears precisely once in the `XML`.",
            "type": "boolean"
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
    let ns: Option<&str> = None;
    let tree = xmltree::Element::parse(xml).unwrap();
    let types = xmltree::Element::parse(xml).unwrap();
    let s = Field::try_from((
        ns,
        &xmltree::XMLNode::Element(tree),
        &vec![(ns, xmltree::XMLNode::Element(types))],
    ))
    .unwrap();
let value: openapiv3::ReferenceOr<openapiv3::Schema> = openapiv3::ReferenceOr::from(&s);
    assert_eq!(
        serde_json::to_value(value).unwrap(),
        json!({
            "description": "A field that is meant to represent a URL.",
            "format": "uri",
            "type": "string"
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
    let ns: Option<&str> = None;
    let tree = xmltree::Element::parse(xml).unwrap();
    let types = xmltree::Element::parse(xml).unwrap();
    let s = Field::try_from((
        ns,
        &xmltree::XMLNode::Element(tree),
        &vec![(ns, xmltree::XMLNode::Element(types))],
    ))
    .unwrap();
let value: openapiv3::ReferenceOr<openapiv3::Schema> = openapiv3::ReferenceOr::from(&s);
    assert_eq!(
        serde_json::to_value(value).unwrap(),
        json!({
            "description": "A field that represents a double precision float",
            "format": "double",
            "type": "number"
        })
    );
}

#[test]
fn test_long_into_schema() {
    let xml: &[u8] = br#"
    <xs:element xmlns:xs="http://www.w3.org/2001/XMLSchema" name="BaseField" type="xs:long">
        <xs:annotation>
            <xs:documentation source="modifiable">none</xs:documentation>
            <xs:documentation xml:lang="en">
                A field that represents 64 bit signed integer
            </xs:documentation>
            <xs:documentation source="required">true</xs:documentation>
        </xs:annotation>
    </xs:element>
    "#;
    let ns: Option<&str> = None;
    let tree = xmltree::Element::parse(xml).unwrap();
    let types = xmltree::Element::parse(xml).unwrap();
    let s = Field::try_from((
        ns,
        &xmltree::XMLNode::Element(tree),
        &vec![(ns, xmltree::XMLNode::Element(types))],
    ))
    .unwrap();
let value: openapiv3::ReferenceOr<openapiv3::Schema> = openapiv3::ReferenceOr::from(&s);
    assert_eq!(
        serde_json::to_value(value).unwrap(),
        json!({
            "description": "A field that represents 64 bit signed integer",
            "format": "int64",
            "type": "integer"
        })
    );
}

#[test]
fn test_datetime_into_schema() {
    let xml: &[u8] = br#"
    <xs:element xmlns:xs="http://www.w3.org/2001/XMLSchema" name="BaseField" type="xs:dateTime">
        <xs:annotation>
            <xs:documentation source="modifiable">none</xs:documentation>
            <xs:documentation xml:lang="en">
                A field that represents date time in ISO 8601 which is basically RFC 3339.
            </xs:documentation>
            <xs:documentation source="required">true</xs:documentation>
        </xs:annotation>
    </xs:element>
    "#;
    let ns: Option<&str> = None;
    let tree = xmltree::Element::parse(xml).unwrap();
    let types = xmltree::Element::parse(xml).unwrap();
    let s = Field::try_from((
        ns,
        &xmltree::XMLNode::Element(tree),
        &vec![(ns, xmltree::XMLNode::Element(types))],
    ))
    .unwrap();
let value: openapiv3::ReferenceOr<openapiv3::Schema> = openapiv3::ReferenceOr::from(&s);
    assert_eq!(
        serde_json::to_value(value).unwrap(),
        json!({
            "description": "A field that represents date time in ISO 8601 which is basically RFC 3339.",
            "format": "date-time",
            "type": "string"
        })
    );
}

#[test]
fn test_base64_binary_into_schema() {
    let xml: &[u8] = br#"
    <xs:element xmlns:xs="http://www.w3.org/2001/XMLSchema" name="BaseField" type="xs:base64Binary">
        <xs:annotation>
            <xs:documentation source="modifiable">none</xs:documentation>
            <xs:documentation xml:lang="en">
                Base64 binary data
            </xs:documentation>
            <xs:documentation source="required">true</xs:documentation>
        </xs:annotation>
    </xs:element>
    "#;
    let ns: Option<&str> = None;
    let tree = xmltree::Element::parse(xml).unwrap();
    let types = xmltree::Element::parse(xml).unwrap();
    let s = Field::try_from((
        ns,
        &xmltree::XMLNode::Element(tree),
        &vec![(ns, xmltree::XMLNode::Element(types))],
    ))
    .unwrap();
let value: openapiv3::ReferenceOr<openapiv3::Schema> = openapiv3::ReferenceOr::from(&s);
    assert_eq!(
        serde_json::to_value(value).unwrap(),
        json!({
            "description": "Base64 binary data",
            "format": "byte",
            "type": "string"
        })
    );
}

#[test]
fn test_normalized_string_into_schema() {
    let xml: &[u8] = br#"
    <xs:element xmlns:xs="http://www.w3.org/2001/XMLSchema" name="BaseField" type="xs:normalizedString">
        <xs:annotation>
            <xs:documentation source="modifiable">none</xs:documentation>
            <xs:documentation xml:lang="en">
                Field that cannot contain new lines
            </xs:documentation>
            <xs:documentation source="required">true</xs:documentation>
        </xs:annotation>
    </xs:element>
    "#;
    let ns: Option<&str> = None;
    let tree = xmltree::Element::parse(xml).unwrap();
    let types = xmltree::Element::parse(xml).unwrap();
    let s = Field::try_from((
        ns,
        &xmltree::XMLNode::Element(tree),
        &vec![(ns, xmltree::XMLNode::Element(types))],
    ))
    .unwrap();
let value: openapiv3::ReferenceOr<openapiv3::Schema> = openapiv3::ReferenceOr::from(&s);
    assert_eq!(
        serde_json::to_value(value).unwrap(),
        json!({
            "description": "Field that cannot contain new lines",
            "type": "string"
        })
    );
}

#[test]
fn test_short_into_schema() {
    let xml: &[u8] = br#"
    <xs:element xmlns:xs="http://www.w3.org/2001/XMLSchema" name="BaseField" type="xs:short">
        <xs:annotation>
            <xs:documentation source="modifiable">none</xs:documentation>
            <xs:documentation xml:lang="en">
                Field that is a 16 bit signed integer
            </xs:documentation>
            <xs:documentation source="required">true</xs:documentation>
        </xs:annotation>
    </xs:element>
    "#;
    let ns: Option<&str> = None;
    let tree = xmltree::Element::parse(xml).unwrap();
    let types = xmltree::Element::parse(xml).unwrap();
    let s = Field::try_from((
        ns,
        &xmltree::XMLNode::Element(tree),
        &vec![(ns, xmltree::XMLNode::Element(types))],
    ))
    .unwrap();
let value: openapiv3::ReferenceOr<openapiv3::Schema> = openapiv3::ReferenceOr::from(&s);
    assert_eq!(
        serde_json::to_value(value).unwrap(),
        json!({
            "description": "Field that is a 16 bit signed integer",
            "type": "integer"
        })
    );
}

#[test]
fn test_decimal_into_schema() {
    let xml: &[u8] = br#"
    <xs:element xmlns:xs="http://www.w3.org/2001/XMLSchema" name="BaseField" type="xs:decimal">
        <xs:annotation>
            <xs:documentation source="modifiable">none</xs:documentation>
            <xs:documentation xml:lang="en">
                Field that is a precise decimal number
            </xs:documentation>
            <xs:documentation source="required">true</xs:documentation>
        </xs:annotation>
    </xs:element>
    "#;
    let ns: Option<&str> = None;
    let tree = xmltree::Element::parse(xml).unwrap();
    let types = xmltree::Element::parse(xml).unwrap();
    let s = Field::try_from((
        ns,
        &xmltree::XMLNode::Element(tree),
        &vec![(ns, xmltree::XMLNode::Element(types))],
    ))
    .unwrap();
    let value: openapiv3::ReferenceOr<openapiv3::Schema> = openapiv3::ReferenceOr::from(&s);
    assert_eq!(
        serde_json::to_value(value).unwrap(),
        json!({
            "description": "Field that is a precise decimal number",
            "type": "string", // verify this!
        })
    );
}

#[test]
fn test_float_into_schema() {
    let xml: &[u8] = br#"
    <xs:element xmlns:xs="http://www.w3.org/2001/XMLSchema" name="BaseField" type="xs:float">
        <xs:annotation>
            <xs:documentation source="modifiable">none</xs:documentation>
            <xs:documentation xml:lang="en">
                Field that is a 32 bit signed floating point type
            </xs:documentation>
            <xs:documentation source="required">true</xs:documentation>
        </xs:annotation>
    </xs:element>
    "#;
    let ns: Option<&str> = None;
    let tree = xmltree::Element::parse(xml).unwrap();
    let types = xmltree::Element::parse(xml).unwrap();
    let s = Field::try_from((
        ns,
        &xmltree::XMLNode::Element(tree),
        &vec![(ns, xmltree::XMLNode::Element(types))],
    ))
    .unwrap();
let value: openapiv3::ReferenceOr<openapiv3::Schema> = openapiv3::ReferenceOr::from(&s);
    assert_eq!(
        serde_json::to_value(value).unwrap(),
        json!({
            "description": "Field that is a 32 bit signed floating point type",
            "type": "number",
            "format": "float"
        })
    );
}

#[test]
fn test_hex_binary_into_schema() {
    let xml: &[u8] = br#"
    <xs:element xmlns:xs="http://www.w3.org/2001/XMLSchema" name="BaseField" type="xs:hexBinary">
        <xs:annotation>
            <xs:documentation source="modifiable">none</xs:documentation>
            <xs:documentation xml:lang="en">
                Hexadecimal binary data
            </xs:documentation>
            <xs:documentation source="required">true</xs:documentation>
        </xs:annotation>
    </xs:element>
    "#;
    let ns: Option<&str> = None;
    let tree = xmltree::Element::parse(xml).unwrap();
    let types = xmltree::Element::parse(xml).unwrap();
    let s = Field::try_from((
        ns,
        &xmltree::XMLNode::Element(tree),
        &vec![(ns, xmltree::XMLNode::Element(types))],
    ))
    .unwrap();
    let value: openapiv3::ReferenceOr<openapiv3::Schema> = openapiv3::ReferenceOr::from(&s);
    assert_eq!(
        serde_json::to_value(value).unwrap(),
        json!({
            "description": "Hexadecimal binary data",
            "type": "string"
        })
    );
}

#[test]
fn test_integer_into_schema() {
    let xml: &[u8] = br#"
    <xs:element xmlns:xs="http://www.w3.org/2001/XMLSchema" name="BaseField" type="xs:integer">
        <xs:annotation>
            <xs:documentation source="modifiable">none</xs:documentation>
            <xs:documentation xml:lang="en">
                Unbounded signed integer
            </xs:documentation>
            <xs:documentation source="required">true</xs:documentation>
        </xs:annotation>
    </xs:element>
    "#;
    let ns: Option<&str> = None;
    let tree = xmltree::Element::parse(xml).unwrap();
    let types = xmltree::Element::parse(xml).unwrap();
    let s = Field::try_from((
        ns,
        &xmltree::XMLNode::Element(tree),
        &vec![(ns, xmltree::XMLNode::Element(types))],
    ))
    .unwrap();
    let value: openapiv3::ReferenceOr<openapiv3::Schema> = openapiv3::ReferenceOr::from(&s);
    assert_eq!(
        serde_json::to_value(value).unwrap(),
        json!({
            "description": "Unbounded signed integer",
            "type": "integer"
        })
    );
}

#[test]
fn test_any_type_into_schema() {
    let xml: &[u8] = br#"
    <xs:element xmlns:xs="http://www.w3.org/2001/XMLSchema" name="BaseField" type="xs:anyType">
        <xs:annotation>
            <xs:documentation source="modifiable">none</xs:documentation>
            <xs:documentation xml:lang="en">
                A field that could be anything
            </xs:documentation>
            <xs:documentation source="required">true</xs:documentation>
        </xs:annotation>
    </xs:element>
    "#;
    let ns: Option<&str> = None;
    let tree = xmltree::Element::parse(xml).unwrap();
    let types = xmltree::Element::parse(xml).unwrap();
    let s = Field::try_from((
        ns,
        &xmltree::XMLNode::Element(tree),
        &vec![(ns, xmltree::XMLNode::Element(types))],
    ))
    .unwrap();
    let value: openapiv3::ReferenceOr<openapiv3::Schema> = openapiv3::ReferenceOr::from(&s);
    assert_eq!(
        serde_json::to_value(value).unwrap(),
        json!({
            "description": "A field that could be anything",
            "type": "string"
        })
    );
}

#[test]
fn test_element_with_simple_type() {
    let xml: &[u8] = br#"
    <xs:element xmlns:xs="http://www.w3.org/2001/XMLSchema" name="BaseField">
        <xs:annotation>
            <xs:documentation source="modifiable">none</xs:documentation>
            <xs:documentation xml:lang="en">
                String with pattern
            </xs:documentation>
            <xs:documentation source="required">true</xs:documentation>
        </xs:annotation>
        <xs:simpleType>
            <xs:restriction base="xs:string">
                <xs:pattern value="pattern"/>
            </xs:restriction>
        </xs:simpleType>
    </xs:element>
    "#;
    let ns: Option<&str> = None;
    let tree = xmltree::Element::parse(xml).unwrap();
    let types = xmltree::Element::parse(xml).unwrap();
    let s = Field::try_from((
        ns,
        &xmltree::XMLNode::Element(tree),
        &vec![(ns, xmltree::XMLNode::Element(types))],
    ))
    .unwrap();
    let value: openapiv3::ReferenceOr<openapiv3::Schema> = openapiv3::ReferenceOr::from(&s);
    assert_eq!(
        serde_json::to_value(value).unwrap(),
        json!({
            "description": "String with pattern",
            "type": "string",
            "pattern": "pattern"
        })
    );
}

#[test]
fn test_attribute_with_simple_type() {
    let xml: &[u8] = br#"
    <xs:attribute xmlns:xs="http://www.w3.org/2001/XMLSchema" name="robotName" use="required">
        <xs:simpleType>
            <xs:restriction base="xs:string">
                <xs:pattern value="[A-Z]-?[0-9]-?[A-Z]-?[0-9]"/>
            </xs:restriction>
        </xs:simpleType>
    </xs:attribute>
    "#;
    let ns: Option<&str> = None;
    let tree = xmltree::Element::parse(xml).unwrap();
    let types = xmltree::Element::parse(xml).unwrap();
    let s = Field::try_from((
        ns,
        &xmltree::XMLNode::Element(tree),
        &vec![(ns, xmltree::XMLNode::Element(types))],
    ))
    .unwrap();
    let value: openapiv3::ReferenceOr<openapiv3::Schema> = openapiv3::ReferenceOr::from(&s);
    assert_eq!(
        serde_json::to_value(value).unwrap(),
        json!({
            "type": "string",
            "pattern": "[A-Z]-?[0-9]-?[A-Z]-?[0-9]"
        })
    );
}
