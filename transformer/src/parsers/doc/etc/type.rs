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
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, PartialEq)]
pub(super) enum Type {
    ObjectType(ObjectType),
    SimpleType(SimpleType),
}

#[derive(Debug, PartialEq)]
pub(super) struct ObjectType {
    pub(super) annotation: Annotation,
    pub(super) name: String,
    pub(super) sequence_elements: Vec<SequenceElement>,
    pub(super) parent: Option<String>,
}

#[derive(Debug, PartialEq)]
pub(super) struct SimpleType {
    pub(super) annotation: Option<Annotation>,
    pub(super) name: String,
    pub(super) pattern: Option<String>,
    pub(super) list: Option<String>,
    pub(super) parent: BaseType,
    pub(super) enumeration: Vec<String>,
    pub(super) min_inclusive: Option<String>,
}

#[derive(Error, Debug, PartialEq)]
pub enum TypeParseError {
    #[error("not a complex or simple type node")]
    NotTypeNode,
    #[error("missing name attribute")]
    MissingName,
    #[error("missing annotation element")]
    MissingAnnotation,
    #[error("failure to parse BaseType")]
    BaseTypeParseError(#[from] ParseBaseTypeError),
    #[error("Missing base attribute")]
    MissingBase,
}

#[derive(Debug, PartialEq)]
pub enum BaseType {
    AnyType,
    AnyUri,
    Base64Binary,
    Boolean,
    DateTime,
    Double,
    HexBinary,
    Int,
    Integer,
    Long,
    String,
}

#[derive(Error, Debug, PartialEq)]
pub enum ParseBaseTypeError {
    #[error("No match for input: `{0}`")]
    NoMatch(String),
}

impl FromStr for BaseType {
    type Err = ParseBaseTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "xs:anyType" => BaseType::AnyType,
            "xs:anyURI" => BaseType::AnyUri,
            "xs:base64Binary" => BaseType::Base64Binary,
            "xs:boolean" => BaseType::Boolean,
            "xs:dateTime" => BaseType::DateTime,
            "xs:double" => BaseType::Double,
            "xs:hexBinary" => BaseType::HexBinary,
            "xs:int" => BaseType::Int,
            "xs:integer" => BaseType::Integer,
            "xs:long" => BaseType::Long,
            "xs:string" => BaseType::String,
            _ => return Err(ParseBaseTypeError::NoMatch(s.to_owned())),
        })
    }
}

impl TryFrom<&xmltree::XMLNode> for Type {
    type Error = TypeParseError;
    fn try_from(value: &xmltree::XMLNode) -> Result<Self, Self::Error> {
        match ObjectType::try_from(value) {
            Err(TypeParseError::NotTypeNode) => Ok(Type::SimpleType(SimpleType::try_from(value)?)),
            Ok(object) => Ok(Type::ObjectType(object)),
            Err(e) => Err(e),
        }
    }
}

impl TryFrom<&xmltree::XMLNode> for SimpleType {
    type Error = TypeParseError;

    fn try_from(value: &xmltree::XMLNode) -> Result<Self, Self::Error> {
        match value {
            xmltree::XMLNode::Element(xmltree::Element {
                namespace: Some(namespace),
                name,
                attributes,
                children,
                ..
            }) if namespace == XML_SCHEMA_NS && name == "simpleType" => {
                let name = attributes
                    .get("name")
                    .ok_or(TypeParseError::MissingName)?
                    .clone();
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
                            children,
                            ..
                        }) if namespace == XML_SCHEMA_NS && node_name == "restriction" => {
                            let parent =
                                attributes.get("base").ok_or(TypeParseError::MissingBase)?;
                            let enumeration = children
                                .iter()
                                .filter_map(|child| match child {
                                    xmltree::XMLNode::Element(xmltree::Element {
                                        namespace: Some(namespace),
                                        name,
                                        attributes,
                                        ..
                                    }) if namespace == XML_SCHEMA_NS && name == "enumeration" => {
                                        attributes.get("value").cloned()
                                    }
                                    _ => None,
                                })
                                .collect();
                            return Ok(Self {
                                annotation,
                                name: name.clone(),
                                enumeration,
                                list: None,
                                min_inclusive: None,
                                parent: parent.parse()?,
                                pattern: None,
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
                let mut sequence_elements = Vec::new();
                let mut parent = None;
                sequence_elements.extend(children.iter().flat_map(SequenceElement::try_from));
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
                Ok(ObjectType {
                    name,
                    annotation,
                    sequence_elements,
                    parent,
                })
            }
            _ => Err(TypeParseError::NotTypeNode),
        }
    }
}

impl From<&Type> for openapiv3::Schema {
    fn from(t: &Type) -> Self {
        match t {
            Type::ObjectType(c) => Self::from(c),
            Type::SimpleType(s) => Self::from(s),
        }
    }
}

impl From<&ObjectType> for openapiv3::Schema {
    fn from(c: &ObjectType) -> Self {
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

impl From<&SimpleType> for openapiv3::Schema {
    fn from(t: &SimpleType) -> Self {
        let schema_data = openapiv3::SchemaData {
            deprecated: t.annotation.as_ref().map(|a| a.deprecated).unwrap_or(false),
            title: Some(t.name.clone()),
            description: t.annotation.as_ref().map(|a| a.description.clone()),
            ..Default::default()
        };

        let r#type = match &t.parent {
            BaseType::AnyType | BaseType::HexBinary | BaseType::String => {
                openapiv3::Type::String(openapiv3::StringType {
                    enumeration: t.enumeration.clone(),
                    pattern: t.pattern.clone(),
                    ..Default::default()
                })
            }
            BaseType::AnyUri => openapiv3::Type::String(openapiv3::StringType {
                enumeration: t.enumeration.clone(),
                pattern: t.pattern.clone(),
                format: openapiv3::VariantOrUnknownOrEmpty::Unknown("uri".to_owned()),
                ..Default::default()
            }),
            BaseType::Base64Binary => openapiv3::Type::String(openapiv3::StringType {
                enumeration: t.enumeration.clone(),
                pattern: t.pattern.clone(),
                format: openapiv3::VariantOrUnknownOrEmpty::Item(openapiv3::StringFormat::Byte),
                ..Default::default()
            }),
            BaseType::Boolean => openapiv3::Type::Boolean {},
            BaseType::DateTime => openapiv3::Type::String(openapiv3::StringType {
                enumeration: t.enumeration.clone(),
                pattern: t.pattern.clone(),
                format: openapiv3::VariantOrUnknownOrEmpty::Item(openapiv3::StringFormat::DateTime),
                ..Default::default()
            }),
            BaseType::Double => openapiv3::Type::Number(openapiv3::NumberType {
                format: openapiv3::VariantOrUnknownOrEmpty::Item(openapiv3::NumberFormat::Double),
                minimum: t.min_inclusive.as_ref().and_then(|m| m.parse().ok()),
                enumeration: t
                    .enumeration
                    .iter()
                    .filter_map(|s| s.parse().ok())
                    .collect(),
                ..Default::default()
            }),
            BaseType::Int => openapiv3::Type::Integer(openapiv3::IntegerType {
                format: openapiv3::VariantOrUnknownOrEmpty::Item(openapiv3::IntegerFormat::Int32),
                minimum: t.min_inclusive.as_ref().and_then(|m| m.parse().ok()),
                enumeration: t
                    .enumeration
                    .iter()
                    .filter_map(|s| s.parse().ok())
                    .collect(),
                ..Default::default()
            }),
            BaseType::Integer => openapiv3::Type::Integer(openapiv3::IntegerType {
                minimum: t.min_inclusive.as_ref().and_then(|m| m.parse().ok()),
                enumeration: t
                    .enumeration
                    .iter()
                    .filter_map(|s| s.parse().ok())
                    .collect(),
                ..Default::default()
            }),
            BaseType::Long => openapiv3::Type::Integer(openapiv3::IntegerType {
                format: openapiv3::VariantOrUnknownOrEmpty::Item(openapiv3::IntegerFormat::Int64),
                minimum: t.min_inclusive.as_ref().and_then(|m| m.parse().ok()),
                enumeration: t
                    .enumeration
                    .iter()
                    .filter_map(|s| s.parse().ok())
                    .collect(),
                ..Default::default()
            }),
        };

        let schema_kind = openapiv3::SchemaKind::Type(r#type);

        Self {
            schema_data,
            schema_kind,
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
        Type::try_from(&xmltree::XMLNode::Element(tree)),
        Ok(Type::ObjectType(ObjectType {
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
        }))
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
        Type::try_from(&xmltree::XMLNode::Element(tree)),
        Ok(Type::ObjectType(ObjectType {
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
        }))
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
        Type::try_from(&xmltree::XMLNode::Element(tree)),
        Ok(Type::ObjectType(ObjectType {
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
        }))
    );
}

#[test]
fn parse_type_that_is_attribute_but_not_extension_test() {
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
        <xs:attribute name="requiredAttribute" type="xs:string" use="required">
            <xs:annotation>
                <xs:documentation source="modifiable">none</xs:documentation>
                <xs:documentation>
                    A field that comes from an attribute.
                </xs:documentation>
                <xs:documentation source="required">true</xs:documentation>
            </xs:annotation>
        </xs:attribute>
    </xs:complexType>
    "#;
    let tree = xmltree::Element::parse(xml).unwrap();
    assert_eq!(
        Type::try_from(&xmltree::XMLNode::Element(tree)),
        Ok(Type::ObjectType(ObjectType {
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
            parent: None
        }))
    );
}

#[test]
fn simple_type_into_schema_test() {
    let xml: &[u8] = br#"
    <xs:simpleType name="CoinType" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:meta="http://www.vmware.com/vcloud/meta">
        <xs:annotation>
            <xs:appinfo><meta:version added-in="5.6"/></xs:appinfo>
            <xs:documentation source="since">5.6</xs:documentation>
            <xs:documentation xml:lang="en">
                An enumeration of the sides of a coin
            </xs:documentation>
        </xs:annotation>
        <xs:restriction base="xs:string">
            <xs:enumeration value="Heads"/>
            <xs:enumeration value="Tails"/>
        </xs:restriction>
    </xs:simpleType>
    "#;

    let tree = xmltree::Element::parse(xml).unwrap();
    let c = Type::try_from(&xmltree::XMLNode::Element(tree)).unwrap();
    let value = openapiv3::Schema::from(&c);
    assert_eq!(
        serde_json::to_value(value).unwrap(),
        json!({
            "description": "An enumeration of the sides of a coin",
            "type": "string",
            "enum": ["Heads", "Tails"],
            "title": "CoinType"
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
    let c = Type::try_from(&xmltree::XMLNode::Element(tree)).unwrap();
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
    let c = Type::try_from(&xmltree::XMLNode::Element(tree)).unwrap();
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
